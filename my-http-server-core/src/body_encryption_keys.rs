use std::sync::{Arc, PoisonError, RwLock};

use base64::Engine;
use zeroize::Zeroizing;

use crate::aes128gcm::{self, OpenError};

/// A key for `Content-Encoding: aes128gcm` bodies: 32 random bytes, and the id the application
/// knows it by. Make one with `openssl rand -base64 32`.
///
/// Cloning it is cheap and does not copy the bytes: every clone shares them, and they are zeroed
/// once the last clone is dropped. `Debug` shows the id, never the bytes.
#[derive(Clone)]
pub struct BodyEncryptionKey {
    inner: Arc<BodyEncryptionKeyInner>,
}

struct BodyEncryptionKeyInner {
    id: String,
    bytes: Zeroizing<[u8; BodyEncryptionKey::LEN]>,
}

impl BodyEncryptionKey {
    /// The length of a key, in bytes.
    pub const LEN: usize = 32;

    pub fn new(id: impl Into<String>, bytes: [u8; Self::LEN]) -> Self {
        Self::from_zeroizing(id.into(), Zeroizing::new(bytes))
    }

    /// The key written as standard base64, 44 characters. Whitespace around it is not part of it:
    /// a key pasted from a password manager often ends with a line break.
    pub fn from_base64(id: impl Into<String>, text: &str) -> Result<Self, String> {
        let id = id.into();

        let decoded = base64::engine::general_purpose::STANDARD
            .decode(text.trim())
            .map(Zeroizing::new)
            .map_err(|_| not_a_key(&id))?;

        if decoded.len() != Self::LEN {
            return Err(not_a_key(&id));
        }

        let mut bytes = Zeroizing::new([0u8; Self::LEN]);
        bytes.copy_from_slice(&decoded);

        Ok(Self::from_zeroizing(id, bytes))
    }

    fn from_zeroizing(id: String, bytes: Zeroizing<[u8; Self::LEN]>) -> Self {
        Self {
            inner: Arc::new(BodyEncryptionKeyInner { id, bytes }),
        }
    }

    pub fn get_id(&self) -> &str {
        &self.inner.id
    }

    /// The bytes as the input keying material (`IKM`) of [`aes128gcm`].
    pub(crate) fn as_ikm(&self) -> &[u8] {
        &self.inner.bytes[..]
    }
}

impl std::fmt::Debug for BodyEncryptionKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("BodyEncryptionKey")
            .field("id", &self.inner.id)
            .finish_non_exhaustive()
    }
}

fn not_a_key(id: &str) -> String {
    format!(
        "Body encryption key '{id}' is not base64 of {} bytes: make one with `openssl rand -base64 32`",
        BodyEncryptionKey::LEN
    )
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

    /// `body` decrypted with the first key that opens it, the keys tried in their order. With no
    /// keys at all nothing opens it: [`OpenError::WrongKeyOrDamaged`].
    pub fn decrypt(&self, body: &[u8]) -> Result<Vec<u8>, OpenError> {
        decrypt_with_any(&self.snapshot(), body).map(|(plain, _)| plain)
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

/// `body` decrypted with the first of `keys` that opens it, and that key.
///
/// Only a first record that does not open sends it on to the next key. A body that is not
/// aes128gcm at all, or one whose first record opened and which broke after it, is refused on the
/// spot: no other key would open what is left either.
pub(crate) fn decrypt_with_any<'k>(
    keys: &'k [BodyEncryptionKey],
    body: &[u8],
) -> Result<(Vec<u8>, &'k BodyEncryptionKey), OpenError> {
    for key in keys {
        match aes128gcm::decrypt(key.as_ikm(), body) {
            Ok(plain) => return Ok((plain, key)),
            Err(OpenError::WrongKeyOrDamaged) => continue,
            Err(err) => return Err(err),
        }
    }

    Err(OpenError::WrongKeyOrDamaged)
}

/// How a request body came encrypted. The server fills it in when it decrypts a body sent with
/// `Content-Encoding: aes128gcm`, and whoever needs it reads it from the request:
/// [`HttpRequest::get_encryption_info`](crate::HttpRequest::get_encryption_info).
///
/// The salt, the record size and the key id are not HTTP headers. They are the first bytes of
/// the body (RFC 8188 §2.1), so once the body is decrypted, this is the only place they are kept.
#[derive(Debug, Clone)]
pub struct EncryptionInfo {
    /// The key of the server's list that opened the body. To answer with the key the client used,
    /// hand it to [`HttpResultBuilder::encode`](crate::HttpResultBuilder::encode).
    pub key: BodyEncryptionKey,
    /// The salt of the body: 16 random bytes, fresh in every body. Remembering these is how a
    /// replayed body is told from a new one.
    pub salt: [u8; 16],
    /// The record size the body was written with.
    pub record_size: u32,
    /// The key id the sender wrote into the header of the body. Usually empty: the key that opened
    /// the body is [`key`](Self::key) either way.
    pub header_key_id: Vec<u8>,
}

impl EncryptionInfo {
    /// What the header of `body` says, with the key that opened it. `None` for a body without an
    /// aes128gcm header, which no key opens anyway.
    pub(crate) fn read(key: BodyEncryptionKey, body: &[u8]) -> Option<Self> {
        let header = aes128gcm::Header::read(body)?;

        Some(Self {
            key,
            salt: *header.salt,
            record_size: header.record_size as u32,
            header_key_id: header.key_id.to_vec(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn key(id: &str, byte: u8) -> BodyEncryptionKey {
        BodyEncryptionKey::new(id, [byte; BodyEncryptionKey::LEN])
    }

    fn base64_of(bytes: &[u8]) -> String {
        base64::engine::general_purpose::STANDARD.encode(bytes)
    }

    #[test]
    fn a_key_is_base64_of_32_bytes() {
        let parsed =
            BodyEncryptionKey::from_base64("proxy", &format!("  {}\n", base64_of(&[9u8; 32])))
                .unwrap();

        assert_eq!(parsed.get_id(), "proxy");
        assert_eq!(parsed.as_ikm(), &[9u8; 32]);

        let short = base64_of(&[0u8; 31]);
        let long = base64_of(&[0u8; 33]);

        for text in ["", "not base64!", "AAAA", short.as_str(), long.as_str()] {
            let err = BodyEncryptionKey::from_base64("proxy", text).unwrap_err();
            assert!(err.contains("'proxy'"), "{err}");
            assert!(err.contains("openssl rand -base64 32"), "{err}");
        }
    }

    #[test]
    fn debug_shows_the_id_and_not_the_bytes() {
        let text = format!("{:?}", key("proxy", 0xAB));

        assert_eq!(text, r#"BodyEncryptionKey { id: "proxy", .. }"#);
    }

    #[test]
    fn a_key_is_found_by_its_id() {
        let keys = BodyEncryptionKeys::new(vec![key("old", 1), key("new", 2)]);

        assert_eq!(keys.get_by_id("new").unwrap().as_ikm(), &[2u8; 32]);
        assert_eq!(keys.get_by_id("old").unwrap().as_ikm(), &[1u8; 32]);
        assert!(keys.get_by_id("other").is_none());
        assert!(keys.get_by_id("NEW").is_none());

        // The first of two keys that share an id.
        keys.set(vec![key("same", 3), key("same", 4)]);
        assert_eq!(keys.get_by_id("same").unwrap().as_ikm(), &[3u8; 32]);
    }

    #[test]
    fn every_key_is_tried_until_one_opens_the_body() {
        let body = aes128gcm::encrypt(&[2u8; 32], b"hosts: {}\n");

        let keys = BodyEncryptionKeys::default();
        assert!(keys.is_empty());
        assert_eq!(keys.decrypt(&body), Err(OpenError::WrongKeyOrDamaged));

        keys.set(vec![key("a", 1), key("b", 2), key("c", 3)]);
        assert_eq!(keys.len(), 3);
        assert_eq!(keys.decrypt(&body).unwrap(), b"hosts: {}\n");

        keys.set(vec![key("a", 1), key("c", 3)]);
        assert_eq!(keys.decrypt(&body), Err(OpenError::WrongKeyOrDamaged));
    }

    /// A body broken after its first record opened is refused on the spot, with what is wrong
    /// with it - not tried with the remaining keys and reported as "no key opens it".
    #[test]
    fn a_body_that_broke_after_the_right_key_opened_it_says_so() {
        let right = key("right", 2);
        let mut body = aes128gcm::encrypt_with(
            right.as_ikm(),
            &[5u8; 16],
            25,
            b"more than one record of data",
        );
        let last = body.len() - 1;
        body[last] ^= 0x01;

        let keys = BodyEncryptionKeys::new(vec![key("wrong", 1), right, key("another", 3)]);

        assert_eq!(keys.decrypt(&body), Err(OpenError::Damaged));
        assert_eq!(keys.decrypt(b"plain"), Err(OpenError::NotEncrypted));
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

    /// The key that opened the body comes back with it, and the header of the body gives the rest.
    #[test]
    fn the_encryption_info_names_the_key_that_opened_the_body() {
        let right = key("right", 2);
        let body = aes128gcm::encrypt_with(right.as_ikm(), &[5u8; 16], 25, b"data");

        let keys = vec![key("wrong", 1), right, key("another", 3)];
        let (plain, opened_with) = decrypt_with_any(&keys, &body).unwrap();

        assert_eq!(plain, b"data");
        assert_eq!(opened_with.get_id(), "right");

        let info = EncryptionInfo::read(opened_with.clone(), &body).unwrap();

        assert_eq!(info.key.get_id(), "right");
        assert_eq!(info.salt, [5u8; 16]);
        assert_eq!(info.record_size, 25);
        assert!(info.header_key_id.is_empty());
    }

    /// RFC 8188 3.2 writes the key id `a1` into its header. The info keeps it as it was sent;
    /// reading the header takes no key.
    #[test]
    fn the_encryption_info_keeps_the_key_id_of_the_header() {
        let body = base64::engine::general_purpose::URL_SAFE_NO_PAD
            .decode("uNCkWiNYzKTnBN9ji3-qWAAAABkCYTHOG8chz_gnvgOqdGYovxyjuqRyJFjEDyoF1Fvkj6hQPdPHI51OEUKEpgz3SsLWIqS_uA")
            .unwrap();

        let info = EncryptionInfo::read(key("any", 0), &body).unwrap();

        assert_eq!(info.header_key_id, b"a1");
        assert_eq!(info.record_size, 25);
        assert_eq!(&info.salt[..], &body[..16]);

        assert!(EncryptionInfo::read(key("any", 0), b"plain").is_none());
    }
}
