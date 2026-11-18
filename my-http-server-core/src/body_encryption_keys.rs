use std::sync::{Arc, PoisonError, RwLock};

use my_http_utils::encryption::{
    EncryptedBody, HttpBodyDecryptError, HttpBodyEncryptionKey, HttpBodyEncryptionKeyError,
};
use rust_extensions::date_time::DateTimeAsMicroseconds;

/// A key for `Content-Encoding: aes128gcm` bodies with the id the application knows it by: an
/// [`HttpBodyEncryptionKey`] of my-http-utils (32 random bytes, `openssl rand -base64 32`) and a
/// name. It derefs to that key, so `key.encrypt(..)` and `key.decrypt(..)` are the standard's own,
/// and it goes wherever an `&HttpBodyEncryptionKey` does -
/// [`HttpResultBuilder::encode`](crate::HttpResultBuilder::encode) among them.
///
/// Cloning it is cheap and does not copy the key: every clone shares it, and its bytes are wiped
/// once the last clone is dropped. `Debug` shows the id, never the key.
#[derive(Clone)]
pub struct BodyEncryptionKey {
    inner: Arc<BodyEncryptionKeyInner>,
}

struct BodyEncryptionKeyInner {
    id: String,
    key: HttpBodyEncryptionKey,
}

impl BodyEncryptionKey {
    /// The length of a key, in bytes.
    pub const LEN: usize = HttpBodyEncryptionKey::LEN;

    pub fn new(id: impl Into<String>, key: HttpBodyEncryptionKey) -> Self {
        Self {
            inner: Arc::new(BodyEncryptionKeyInner { id: id.into(), key }),
        }
    }

    /// The key written as standard base64, 44 characters. Whitespace around it is not part of it:
    /// a key pasted from a password manager often ends with a line break. The error never repeats
    /// what it was given.
    pub fn from_base64(
        id: impl Into<String>,
        text: &str,
    ) -> Result<Self, HttpBodyEncryptionKeyError> {
        Ok(Self::new(id, HttpBodyEncryptionKey::from_base64(text)?))
    }

    /// Exactly 32 bytes.
    pub fn from_bytes(
        id: impl Into<String>,
        bytes: &[u8],
    ) -> Result<Self, HttpBodyEncryptionKeyError> {
        Ok(Self::new(id, HttpBodyEncryptionKey::from_bytes(bytes)?))
    }

    pub fn get_id(&self) -> &str {
        &self.inner.id
    }
}

impl std::ops::Deref for BodyEncryptionKey {
    type Target = HttpBodyEncryptionKey;

    fn deref(&self) -> &HttpBodyEncryptionKey {
        &self.inner.key
    }
}

impl std::fmt::Debug for BodyEncryptionKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("BodyEncryptionKey")
            .field("id", &self.inner.id)
            .finish_non_exhaustive()
    }
}

/// The keys a request body announced with `Content-Encoding: aes128gcm` is decrypted with.
/// There can be several at once: one per client, or an old one and a new one while a key is
/// being rotated. Each is tried in turn until one opens the body.
///
/// Handed to the server with
/// [`MyHttpServer::set_body_encryption_keys`](crate::MyHttpServer::set_body_encryption_keys), as an
/// `Arc` the application keeps too: [`set`](Self::set) replaces the keys while the server runs, so
/// keys that only arrive after the start are used from the next body read on.
///
/// An action that answers encrypted takes the key it wants by id and hands it to
/// [`HttpResultBuilder::encode`](crate::HttpResultBuilder::encode).
pub struct BodyEncryptionKeys {
    keys: RwLock<Arc<Vec<BodyEncryptionKey>>>,
}

impl BodyEncryptionKeys {
    pub fn new(keys: Vec<BodyEncryptionKey>) -> Self {
        Self {
            keys: RwLock::new(Arc::new(keys)),
        }
    }

    /// Replaces every key. A body that is being decrypted right now finishes with the keys it
    /// started with.
    pub fn set(&self, keys: Vec<BodyEncryptionKey>) {
        *self.keys.write().unwrap_or_else(PoisonError::into_inner) = Arc::new(keys);
    }

    /// The key with this id. Ids are meant to be unique; if two keys share one, the first of them
    /// is given.
    pub fn get_by_id(&self, id: &str) -> Option<BodyEncryptionKey> {
        self.snapshot()
            .iter()
            .find(|key| key.get_id() == id)
            .cloned()
    }

    pub fn len(&self) -> usize {
        self.snapshot().len()
    }

    pub fn is_empty(&self) -> bool {
        self.snapshot().is_empty()
    }

    /// A plain aes128gcm `body` - an answer encrypted with
    /// [`encode`](crate::HttpResultBuilder::encode), say - decrypted with the first key that opens
    /// it, the keys tried in their order: what
    /// [`HttpBodyEncryptionKey::decrypt_with_any`] does with the keys of this list. With no keys
    /// at all nothing opens it: [`HttpBodyDecryptError::WrongKeyOrDamaged`].
    ///
    /// A request body is not plain aes128gcm: it starts with the sender's clock, and the server
    /// opens it as an [`EncryptedBody`].
    pub fn decrypt(&self, body: &[u8]) -> Result<Vec<u8>, HttpBodyDecryptError> {
        try_each_key(&self.snapshot(), |key| key.decrypt(body)).map(|(_, plain)| plain)
    }

    /// The keys as they are now. They are read once per body, so a `set` while a body is being
    /// decrypted does not change the keys it is tried with.
    pub(crate) fn snapshot(&self) -> Arc<Vec<BodyEncryptionKey>> {
        self.keys
            .read()
            .unwrap_or_else(PoisonError::into_inner)
            .clone()
    }
}

impl Default for BodyEncryptionKeys {
    fn default() -> Self {
        Self::new(Vec::new())
    }
}

/// `open` tried with each of `keys` in turn, the way
/// [`HttpBodyEncryptionKey::decrypt_with_any`] does it: only a key that does not open the body
/// sends it on to the next one. A body that is not aes128gcm at all, or that a key opened and
/// that is cut short, malformed or shorter than its clock, is refused on the spot: no other key
/// would do better.
fn try_each_key<T>(
    keys: &[BodyEncryptionKey],
    open: impl Fn(&HttpBodyEncryptionKey) -> Result<T, HttpBodyDecryptError>,
) -> Result<(&BodyEncryptionKey, T), HttpBodyDecryptError> {
    for key in keys {
        match open(key) {
            Ok(opened) => return Ok((key, opened)),
            Err(HttpBodyDecryptError::WrongKeyOrDamaged) => {}
            Err(err) => return Err(err),
        }
    }

    Err(HttpBodyDecryptError::WrongKeyOrDamaged)
}

/// A request body opened as the standard's [`EncryptedBody`] - the salt, the sender's clock and
/// the body - with the first of `keys` that opens it, and that key.
///
/// `bytes` are not empty. Empty bytes are an empty body to the standard, which opens them with
/// any key at all, so the caller hands them on before it gets here.
pub(crate) fn open_request_body<'k>(
    keys: &'k [BodyEncryptionKey],
    bytes: &[u8],
) -> Result<(&'k BodyEncryptionKey, EncryptedBody), HttpBodyDecryptError> {
    debug_assert!(!bytes.is_empty(), "empty bytes open with any key");

    try_each_key(keys, |key| EncryptedBody::decrypt(bytes, key))
}

/// How a request body came encrypted. The server fills it in when it decrypts a body sent with
/// `Content-Encoding: aes128gcm`, and whoever needs it reads it from the request:
/// [`HttpRequest::get_encryption_info`](crate::HttpRequest::get_encryption_info).
///
/// The body is the standard's [`EncryptedBody`]: the salt, the record size and the key id are the
/// first bytes of what came over the wire (RFC 8188 §2.1), and the sender's clock the first 8
/// bytes of what is decrypted. None of them is an HTTP header, so once the body is decrypted, this
/// is the only place they are kept.
#[derive(Debug, Clone)]
pub struct EncryptionInfo {
    /// The key of the server's list that opened the body. To answer with the key the client used,
    /// hand it to [`HttpResultBuilder::encode`](crate::HttpResultBuilder::encode).
    pub key: BodyEncryptionKey,
    /// The salt of the body ([`EncryptedBody::salt`]): 16 random bytes, fresh in every body.
    /// Remembering these is how a replayed body is told from a new one.
    pub salt: [u8; 16],
    /// The record size the body was written with: 4096 from my-http-utils.
    pub record_size: u32,
    /// The key id the sender wrote into the header of the body. Empty from my-http-utils: the key
    /// that opened the body is [`key`](Self::key) either way.
    pub header_key_id: Vec<u8>,
    /// The sender's clock when it encrypted the body ([`EncryptedBody::timestamp`]): unix
    /// microseconds. The server takes it off the body. Checking it against the server's clock is
    /// the action's job: see `docs/http-actions/encrypted-requests.md`.
    pub timestamp: DateTimeAsMicroseconds,
}

impl EncryptionInfo {
    /// `key` opened `bytes` - the body as it came over the wire - into `salt` and `timestamp`;
    /// the header of `bytes` gives the rest.
    pub(crate) fn new(
        key: BodyEncryptionKey,
        bytes: &[u8],
        salt: [u8; 16],
        timestamp: DateTimeAsMicroseconds,
    ) -> Self {
        let (record_size, header_key_id) = read_header(bytes);

        Self {
            key,
            salt,
            record_size,
            header_key_id: header_key_id.to_vec(),
            timestamp,
        }
    }
}

/// The record size and the key id of an aes128gcm header:
/// `salt (16) || rs (u32, big-endian) || idlen (1) || keyid (idlen)`. Read from a body a key has
/// opened, so the header is there; a header cut short reads as `0` and no key id.
fn read_header(bytes: &[u8]) -> (u32, &[u8]) {
    const RECORD_SIZE_AT: usize = 16;
    const KEY_ID_LEN_AT: usize = RECORD_SIZE_AT + 4;

    let record_size = bytes
        .get(RECORD_SIZE_AT..KEY_ID_LEN_AT)
        .and_then(|rs| rs.try_into().ok())
        .map(u32::from_be_bytes)
        .unwrap_or(0);

    let key_id = bytes
        .get(KEY_ID_LEN_AT)
        .and_then(|len| bytes.get(KEY_ID_LEN_AT + 1..KEY_ID_LEN_AT + 1 + *len as usize))
        .unwrap_or(&[]);

    (record_size, key_id)
}

#[cfg(test)]
mod tests {
    use super::*;

    use HttpBodyDecryptError::*;

    fn http_key(byte: u8) -> HttpBodyEncryptionKey {
        HttpBodyEncryptionKey::from_bytes(&[byte; HttpBodyEncryptionKey::LEN]).unwrap()
    }

    fn key(id: &str, byte: u8) -> BodyEncryptionKey {
        BodyEncryptionKey::new(id, http_key(byte))
    }

    /// Whether `key` is the key made of `byte`: it opens what that key encrypted.
    fn is_key_of(key: &BodyEncryptionKey, byte: u8) -> bool {
        key.decrypt(&http_key(byte).encrypt(b"probe")).as_deref() == Ok(&b"probe"[..])
    }

    /// The standard key `AAEC...Hh8=`: the bytes 0 to 31.
    const STANDARD_KEY: &str = "AAECAwQFBgcICQoLDA0ODxAREhMUFRYXGBkaGxwdHh8=";

    #[test]
    fn a_key_is_base64_of_32_bytes() {
        let parsed =
            BodyEncryptionKey::from_base64("proxy", &format!("  {STANDARD_KEY}\n")).unwrap();
        assert_eq!(parsed.get_id(), "proxy");

        let bytes: Vec<u8> = (0..32).collect();
        let body = HttpBodyEncryptionKey::from_bytes(&bytes)
            .unwrap()
            .encrypt(b"data");
        assert_eq!(parsed.decrypt(&body).unwrap(), b"data");

        assert_eq!(
            BodyEncryptionKey::from_base64("proxy", "not base64!").unwrap_err(),
            HttpBodyEncryptionKeyError::NotBase64
        );
        // Base64 of 3 bytes.
        assert_eq!(
            BodyEncryptionKey::from_base64("proxy", "AAAA").unwrap_err(),
            HttpBodyEncryptionKeyError::WrongLength { len: 3 }
        );
        assert_eq!(
            BodyEncryptionKey::from_bytes("proxy", &[0; 31]).unwrap_err(),
            HttpBodyEncryptionKeyError::WrongLength { len: 31 }
        );
        assert!(is_key_of(
            &BodyEncryptionKey::from_bytes("x", &[6; 32]).unwrap(),
            6
        ));
    }

    #[test]
    fn debug_shows_the_id_and_not_the_key() {
        let text = format!("{:?}", key("proxy", 0xAB));

        assert_eq!(text, r#"BodyEncryptionKey { id: "proxy", .. }"#);
    }

    #[test]
    fn a_key_is_found_by_its_id() {
        let keys = BodyEncryptionKeys::new(vec![key("old", 1), key("new", 2)]);

        assert!(is_key_of(&keys.get_by_id("new").unwrap(), 2));
        assert!(is_key_of(&keys.get_by_id("old").unwrap(), 1));
        assert!(keys.get_by_id("other").is_none());
        assert!(keys.get_by_id("NEW").is_none());

        // The first of two keys that share an id.
        keys.set(vec![key("same", 3), key("same", 4)]);
        assert!(is_key_of(&keys.get_by_id("same").unwrap(), 3));
    }

    #[test]
    fn every_key_is_tried_until_one_opens_the_body() {
        let body = http_key(2).encrypt(b"hosts: {}\n");

        let keys = BodyEncryptionKeys::default();
        assert!(keys.is_empty());
        assert_eq!(keys.decrypt(&body), Err(WrongKeyOrDamaged));

        keys.set(vec![key("a", 1), key("b", 2), key("c", 3)]);
        assert_eq!(keys.len(), 3);
        assert_eq!(keys.decrypt(&body).unwrap(), b"hosts: {}\n");

        keys.set(vec![key("a", 1), key("c", 3)]);
        assert_eq!(keys.decrypt(&body), Err(WrongKeyOrDamaged));
    }

    /// A body the right key opened and that turned out cut short is refused on the spot, with
    /// what is wrong with it - not tried with the remaining keys and reported as "no key opens it".
    #[test]
    fn a_body_cut_short_after_the_right_key_opened_it_says_so() {
        let right = key("right", 2);
        let keys = vec![key("wrong", 1), right.clone(), key("another", 3)];

        // Two records; only the first one, whose delimiter says more is coming, is sent.
        let body = EncryptedBody {
            salt: EncryptedBody::random_salt(),
            timestamp: DateTimeAsMicroseconds::now(),
            body: vec![b'x'; 5000],
        }
        .encrypt(&right);
        let cut = &body[..16 + 5 + 4096];

        assert_eq!(open_request_body(&keys, cut).unwrap_err(), CutShort);
        assert_eq!(
            open_request_body(&keys, b"plain").unwrap_err(),
            NotEncrypted
        );
    }

    /// What was encrypted is shorter than the clock: the right key opened it, so that is the
    /// answer, not "no key opens it".
    #[test]
    fn a_body_shorter_than_the_clock_is_no_timestamp() {
        let keys = vec![key("wrong", 1), key("right", 2)];

        let body = http_key(2).encrypt(b"1234567");

        assert_eq!(open_request_body(&keys, &body).unwrap_err(), NoTimestamp);
    }

    /// A body being decrypted keeps the keys it started with.
    #[test]
    fn a_snapshot_is_not_changed_by_a_later_set() {
        let keys = BodyEncryptionKeys::new(vec![key("a", 1)]);

        let snapshot = keys.snapshot();
        keys.set(vec![key("b", 2)]);

        assert_eq!(snapshot.len(), 1);
        assert_eq!(snapshot[0].get_id(), "a");
        assert_eq!(keys.snapshot()[0].get_id(), "b");
    }

    /// The key that opened the body comes back with what the standard decrypts it into, and the
    /// header of the body gives the rest.
    #[test]
    fn the_encryption_info_names_the_key_that_opened_the_body() {
        let right = key("right", 2);
        let sent = EncryptedBody {
            salt: [5u8; 16],
            timestamp: DateTimeAsMicroseconds::new(1_700_000_000_123_456),
            body: b"data".to_vec(),
        };
        let bytes = sent.encrypt(&right);

        let keys = vec![key("wrong", 1), right, key("another", 3)];
        let (opened_with, opened) = open_request_body(&keys, &bytes).unwrap();

        assert_eq!(opened_with.get_id(), "right");
        assert_eq!(opened, sent);

        let info = EncryptionInfo::new(opened_with.clone(), &bytes, opened.salt, opened.timestamp);

        assert_eq!(info.key.get_id(), "right");
        assert_eq!(info.salt, [5u8; 16]);
        assert_eq!(info.record_size, 4096);
        assert!(info.header_key_id.is_empty());
        assert_eq!(info.timestamp.unix_microseconds, 1_700_000_000_123_456);
    }

    /// The format on the wire, put together by hand rather than by `EncryptedBody`: the clock
    /// first, 8 bytes little-endian, then the body.
    #[test]
    fn a_request_body_starts_with_the_clock_of_the_sender() {
        let mut plain = 1_700_000_000_123_456i64.to_le_bytes().to_vec();
        plain.extend_from_slice(br#"{"email":"a@b.com"}"#);

        let keys = vec![key("client", 4)];
        let (_, opened) = open_request_body(&keys, &http_key(4).encrypt(&plain)).unwrap();

        assert_eq!(opened.timestamp.unix_microseconds, 1_700_000_000_123_456);
        assert_eq!(opened.body, br#"{"email":"a@b.com"}"#);

        // The clock alone is an empty body.
        let (_, opened) = open_request_body(&keys, &http_key(4).encrypt(&[0u8; 8])).unwrap();
        assert!(opened.body.is_empty());
    }

    /// RFC 8188 3.2 writes records of 25 bytes and the key id `a1` into its header. Reading them
    /// takes no key.
    #[test]
    fn the_header_gives_the_record_size_and_the_key_id() {
        let mut bytes = vec![7u8; 16];
        bytes.extend_from_slice(&25u32.to_be_bytes());
        bytes.push(2);
        bytes.extend_from_slice(b"a1");
        bytes.extend_from_slice(&[0xAA; 50]);

        assert_eq!(read_header(&bytes), (25, &b"a1"[..]));

        let info = EncryptionInfo::new(
            key("any", 0),
            &bytes,
            [7u8; 16],
            DateTimeAsMicroseconds::new(0),
        );
        assert_eq!(info.record_size, 25);
        assert_eq!(info.header_key_id, b"a1");

        assert_eq!(read_header(b"short"), (0, &b""[..]));
    }
}
