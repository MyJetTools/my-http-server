//! `Content-Encoding: aes128gcm` - [RFC 8188](https://www.rfc-editor.org/rfc/rfc8188),
//! "Encrypted Content-Encoding for HTTP".
//!
//! ```text
//! body   = header || record_0 || record_1 || ... || record_last
//! header = salt (16 random bytes) || rs (uint32, big-endian) || idlen (1 byte) || keyid (idlen bytes)
//! ```
//!
//! - `PRK = HMAC-SHA-256(salt, IKM)`, the IKM being the bytes of the key;
//! - `CEK = HMAC-SHA-256(PRK, "Content-Encoding: aes128gcm" || 0x00 || 0x01)[..16]`;
//! - `NONCE_BASE = HMAC-SHA-256(PRK, "Content-Encoding: nonce" || 0x00 || 0x01)[..12]`, and the
//!   nonce of record `i` is `NONCE_BASE XOR i`, `i` written as a 12-byte big-endian number;
//! - a record is `AES-128-GCM(CEK, nonce, chunk || delimiter || 0x00 padding)`: the delimiter is
//!   `0x01`, or `0x02` for the last record, and every record but the last is exactly `rs` bytes.
//!
//! It is the encoding my-reverse-proxy reads the data of its services in. Its
//! `docs/encrypted-service-data.md` has the test vectors every implementation checks itself
//! against, and the tests below use the same ones.
//!
//! The server uses it on both sides: a request body announced with `Content-Encoding: aes128gcm`
//! is opened with [`BodyEncryptionKeys`](crate::BodyEncryptionKeys), and
//! [`HttpResultBuilder::encode`](crate::HttpResultBuilder::encode) encrypts a response.

use aes_gcm::aead::{Aead, KeyInit, Nonce};
use aes_gcm::Aes128Gcm;
use hmac::{Hmac, Mac};
use sha2::Sha256;
use zeroize::Zeroizing;

/// The value of `Content-Encoding` that says a body is encrypted this way.
pub const CONTENT_ENCODING: &str = "aes128gcm";

/// The record size [`encrypt`] writes: a record carries up to 4079 bytes of data.
pub const RECORD_SIZE: u32 = 4096;

const SALT_LEN: usize = 16;
const CONTENT_KEY_LEN: usize = 16;
const NONCE_LEN: usize = 12;
const TAG_LEN: usize = 16;

/// The salt, the record size and the length of the key id.
const HEADER_LEN: usize = SALT_LEN + 4 + 1;

/// What a record adds to the data it carries: the delimiter and the tag.
const RECORD_OVERHEAD: usize = TAG_LEN + 1;

const DELIMITER: u8 = 0x01;
const LAST_DELIMITER: u8 = 0x02;

/// The info of HKDF-Expand with its counter: the first block of the output is all either takes.
const CONTENT_KEY_INFO: &[u8] = b"Content-Encoding: aes128gcm\x00\x01";
const NONCE_INFO: &[u8] = b"Content-Encoding: nonce\x00\x01";

/// Why a body does not open.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OpenError {
    /// Too short for its header, or a header no aes128gcm body has.
    NotEncrypted,
    /// The first record does not open: the body was encrypted with another key, or it is damaged.
    WrongKeyOrDamaged,
    /// The first record opened and the body broke after it: a later record does not open, or the
    /// records are not laid out the way aes128gcm lays them out. The key is right, the body is not.
    Damaged,
    /// The body ends without its last record: it is cut short.
    CutShort,
}

impl std::fmt::Display for OpenError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let text = match self {
            Self::NotEncrypted => "it is not an aes128gcm body",
            Self::WrongKeyOrDamaged => {
                "no key opens it: it was encrypted with another key, or it is damaged"
            }
            Self::Damaged => "it is damaged",
            Self::CutShort => "it ends without its last record: it is cut short",
        };

        f.write_str(text)
    }
}

/// `plain`, encrypted with `ikm` under a fresh random salt, in records of [`RECORD_SIZE`], with no
/// key id.
///
/// Panics only when the operating system has no random bytes to give for the salt. A body is
/// never sent unencrypted instead.
pub fn encrypt(ikm: &[u8], plain: &[u8]) -> Vec<u8> {
    let mut salt = [0u8; SALT_LEN];

    getrandom::fill(&mut salt).expect("the operating system gives no random bytes for a salt");

    encrypt_with(ikm, &salt, RECORD_SIZE, plain)
}

/// `plain`, encrypted with `ikm` under `salt`, in records of `record_size` bytes, with no key id.
/// A salt is never to be used twice with one key: [`encrypt`] makes a fresh one each time, and
/// this one exists for it and for the test vectors.
pub(crate) fn encrypt_with(
    ikm: &[u8],
    salt: &[u8; SALT_LEN],
    record_size: u32,
    plain: &[u8],
) -> Vec<u8> {
    assert!(
        record_size as usize > RECORD_OVERHEAD,
        "a record of {record_size} bytes carries no data"
    );

    let chunk_len = record_size as usize - RECORD_OVERHEAD;
    let keys = RecordKeys::derive(ikm, salt);

    let mut chunks: Vec<&[u8]> = plain.chunks(chunk_len).collect();

    // Empty data is still a record: the last one, with nothing but its delimiter.
    if chunks.is_empty() {
        chunks.push(&[]);
    }

    let mut body = Vec::with_capacity(HEADER_LEN + plain.len() + chunks.len() * RECORD_OVERHEAD);

    body.extend_from_slice(salt);
    body.extend_from_slice(&record_size.to_be_bytes());
    body.push(0); // no key id

    let last = chunks.len() - 1;

    for (sequence, chunk) in chunks.into_iter().enumerate() {
        let mut record = Vec::with_capacity(chunk.len() + 1);
        record.extend_from_slice(chunk);
        record.push(if sequence == last {
            LAST_DELIMITER
        } else {
            DELIMITER
        });

        let sealed = keys
            .cipher
            .encrypt(&keys.nonce(sequence as u64), record.as_slice())
            .expect("AES-128-GCM encrypts a record of any size this function makes");

        body.extend_from_slice(&sealed);
    }

    body
}

/// The body, decrypted with `ikm`. Either all of it opens or nothing is given: a record that fails
/// its tag, a body cut short and anything after the last record refuse the whole body.
///
/// A wrong key fails on the first record, so trying a key that is not the one costs one record
/// (at most `rs` bytes), never the whole body. A key id in the header is skipped: the key is the
/// caller's to choose.
pub fn decrypt(ikm: &[u8], body: &[u8]) -> Result<Vec<u8>, OpenError> {
    let header = Header::read(body).ok_or(OpenError::NotEncrypted)?;

    let mut records = &body[header.len..];

    if records.is_empty() {
        return Err(OpenError::CutShort);
    }

    let keys = RecordKeys::derive(ikm, header.salt);

    let mut plain = Vec::new();
    let mut sequence = 0u64;

    loop {
        let (record, rest) = records.split_at(records.len().min(header.record_size));

        let Ok(opened) = keys.cipher.decrypt(&keys.nonce(sequence), record) else {
            return Err(if sequence == 0 {
                OpenError::WrongKeyOrDamaged
            } else {
                OpenError::Damaged
            });
        };

        // The padding is zero bytes after the delimiter.
        let Some(delimiter_at) = opened.iter().rposition(|byte| *byte != 0) else {
            return Err(OpenError::Damaged);
        };

        let is_last = rest.is_empty();

        match (opened[delimiter_at], is_last) {
            (LAST_DELIMITER, true) | (DELIMITER, false) => {}
            (DELIMITER, true) => return Err(OpenError::CutShort),
            _ => return Err(OpenError::Damaged),
        }

        if sequence == 0 {
            // Only now that the key is known to be the one: a wrong key costs no allocation the
            // size of the body.
            plain.reserve(records.len());
        }

        plain.extend_from_slice(&opened[..delimiter_at]);

        if is_last {
            return Ok(plain);
        }

        records = rest;
        sequence += 1;
    }
}

/// The header at the start of a body: it travels in the body, not in HTTP headers.
pub(crate) struct Header<'s> {
    pub(crate) salt: &'s [u8; SALT_LEN],
    pub(crate) record_size: usize,
    /// The key id the sender wrote, often empty.
    pub(crate) key_id: &'s [u8],
    /// The header with its key id: where the records start.
    len: usize,
}

impl<'s> Header<'s> {
    pub(crate) fn read(body: &'s [u8]) -> Option<Self> {
        if body.len() < HEADER_LEN {
            return None;
        }

        let salt: &[u8; SALT_LEN] = body[..SALT_LEN].try_into().ok()?;

        let record_size =
            u32::from_be_bytes(body[SALT_LEN..SALT_LEN + 4].try_into().ok()?) as usize;

        // RFC 8188 2.1: a record has to carry at least the delimiter besides its tag.
        if record_size <= RECORD_OVERHEAD {
            return None;
        }

        let len = HEADER_LEN + body[HEADER_LEN - 1] as usize;

        if body.len() < len {
            return None;
        }

        Some(Self {
            salt,
            record_size,
            key_id: &body[HEADER_LEN..len],
            len,
        })
    }
}

struct RecordKeys {
    cipher: Aes128Gcm,
    nonce_base: [u8; NONCE_LEN],
}

impl RecordKeys {
    /// `CEK` and `NONCE_BASE` of RFC 8188: HKDF-Extract with the salt, then HKDF-Expand with the
    /// two infos. Both are shorter than a SHA-256 block, so one HMAC each.
    fn derive(ikm: &[u8], salt: &[u8; SALT_LEN]) -> Self {
        let prk = Zeroizing::new(hmac_sha256(salt, ikm));
        let content_key = Zeroizing::new(hmac_sha256(&prk[..], CONTENT_KEY_INFO));
        let nonce = hmac_sha256(&prk[..], NONCE_INFO);

        let cipher = Aes128Gcm::new_from_slice(&content_key[..CONTENT_KEY_LEN])
            .expect("16 bytes is an AES-128 key");

        let mut nonce_base = [0u8; NONCE_LEN];
        nonce_base.copy_from_slice(&nonce[..NONCE_LEN]);

        Self { cipher, nonce_base }
    }

    /// The nonce of the record `sequence`: the base, with the number XORed into its end.
    fn nonce(&self, sequence: u64) -> Nonce<Aes128Gcm> {
        let mut nonce = self.nonce_base;

        for (byte, sequence_byte) in nonce[NONCE_LEN - 8..]
            .iter_mut()
            .zip(sequence.to_be_bytes())
        {
            *byte ^= sequence_byte;
        }

        Nonce::<Aes128Gcm>::from(nonce)
    }
}

fn hmac_sha256(key: &[u8], message: &[u8]) -> [u8; 32] {
    let mut mac = <Hmac<Sha256> as hmac::KeyInit>::new_from_slice(key)
        .expect("HMAC takes a key of any length");
    mac.update(message);
    mac.finalize().into_bytes().into()
}

#[cfg(test)]
mod tests {
    use base64::Engine;

    use super::*;

    fn base64url(text: &str) -> Vec<u8> {
        base64::engine::general_purpose::URL_SAFE_NO_PAD
            .decode(text)
            .unwrap()
    }

    fn base64(text: &str) -> Vec<u8> {
        base64::engine::general_purpose::STANDARD
            .decode(text)
            .unwrap()
    }

    fn salt_of(body: &[u8]) -> [u8; SALT_LEN] {
        body[..SALT_LEN].try_into().unwrap()
    }

    /// RFC 8188, 3.1: one record, no key id.
    const RFC_3_1_KEY: &str = "yqdlZ-tYemfogSmv7Ws5PQ";
    const RFC_3_1_BODY: &str =
        "I1BsxtFttlv3u_Oo94xnmwAAEAAA-NAVub2qFgBEuQKRapoZu-IxkIva3MEB1PD-ly8Thjg";

    /// RFC 8188, 3.2: records of 25 bytes, the key id `a1`.
    const RFC_3_2_KEY: &str = "BO3ZVPxUlnLORbVGMpbT1Q";
    const RFC_3_2_BODY: &str = "uNCkWiNYzKTnBN9ji3-qWAAAABkCYTHOG8chz_gnvgOqdGYovxyjuqRyJFjEDyoF1Fvkj6hQPdPHI51OEUKEpgz3SsLWIqS_uA";

    /// The vector of my-reverse-proxy's docs/encrypted-service-data.md: a key of 32 bytes (0 to
    /// 31), in standard base64.
    const OUR_KEY: &str = "AAECAwQFBgcICQoLDA0ODxAREhMUFRYXGBkaGxwdHh8=";
    const OUR_BODY: &str = "oKGio6SlpqeoqaqrrK2urwAAEAAAd4lQdR25LBqmiaRlRbryiJTYDikv7OAMtydV1zSRPu4Fy70/BNOmc0q8RjUIqwznMD1/D9zqlMc=";
    const OUR_PLAIN: &[u8] = b"ip_white_lists:\n  office:\n  - 10.0.0.1\n";

    #[test]
    fn the_vectors_decrypt_to_their_plain_text() {
        assert_eq!(
            decrypt(&base64url(RFC_3_1_KEY), &base64url(RFC_3_1_BODY)).unwrap(),
            b"I am the walrus"
        );
        assert_eq!(
            decrypt(&base64url(RFC_3_2_KEY), &base64url(RFC_3_2_BODY)).unwrap(),
            b"I am the walrus"
        );
        assert_eq!(
            decrypt(&base64(OUR_KEY), &base64(OUR_BODY)).unwrap(),
            OUR_PLAIN
        );
    }

    #[test]
    fn the_vectors_are_encrypted_byte_for_byte() {
        let body = base64url(RFC_3_1_BODY);
        assert_eq!(
            encrypt_with(
                &base64url(RFC_3_1_KEY),
                &salt_of(&body),
                4096,
                b"I am the walrus"
            ),
            body
        );

        let body = base64(OUR_BODY);
        assert_eq!(
            encrypt_with(&base64(OUR_KEY), &salt_of(&body), 4096, OUR_PLAIN),
            body
        );
    }

    #[test]
    fn another_key_does_not_open_a_body() {
        assert_eq!(
            decrypt(&base64(OUR_KEY), &base64url(RFC_3_1_BODY)),
            Err(OpenError::WrongKeyOrDamaged)
        );
        assert_eq!(
            decrypt(&base64url(RFC_3_1_KEY), &base64(OUR_BODY)),
            Err(OpenError::WrongKeyOrDamaged)
        );
    }

    #[test]
    fn a_damaged_body_does_not_open() {
        let key = base64(OUR_KEY);
        let body = base64(OUR_BODY);

        for index in [0, 20, body.len() / 2, body.len() - 1] {
            let mut damaged = body.clone();
            damaged[index] ^= 0x01;
            assert!(decrypt(&key, &damaged).is_err(), "byte {index}");
        }

        assert!(decrypt(&key, &body[..body.len() - 1]).is_err());
    }

    /// Past the first record the key is known to be right, so a record that does not open there
    /// is a damaged body, not another key.
    #[test]
    fn a_later_record_that_does_not_open_is_a_damaged_body() {
        let key = base64(OUR_KEY);
        let mut body = encrypt_with(&key, &[7u8; SALT_LEN], 25, b"more than one record of data");

        let last = body.len() - 1;
        body[last] ^= 0x01;

        assert_eq!(decrypt(&key, &body), Err(OpenError::Damaged));
    }

    /// A body of several records that lost its last ones ends with a record that says more is
    /// coming.
    #[test]
    fn a_body_cut_short_does_not_open() {
        let key = base64(OUR_KEY);
        let body = encrypt_with(&key, &[7u8; SALT_LEN], 25, b"more than one record of data");

        assert!(body.len() > HEADER_LEN + 25);

        let cut = &body[..HEADER_LEN + 25];
        assert_eq!(decrypt(&key, cut), Err(OpenError::CutShort));

        // A header and no records at all.
        assert_eq!(decrypt(&key, &body[..HEADER_LEN]), Err(OpenError::CutShort));
    }

    #[test]
    fn what_is_encrypted_opens_whatever_its_length() {
        let key = base64(OUR_KEY);

        for len in [0, 1, 7, 8, 9, 16, 17, 100, 4079, 4080, 10_000] {
            let plain: Vec<u8> = (0..len).map(|index| (index % 251) as u8).collect();

            for record_size in [18, 25, 4096] {
                let body = encrypt_with(&key, &[3u8; SALT_LEN], record_size, &plain);
                assert_eq!(
                    decrypt(&key, &body).unwrap(),
                    plain,
                    "{len} in {record_size}"
                );
            }

            let body = encrypt(&key, &plain);
            assert_eq!(decrypt(&key, &body).unwrap(), plain, "{len}");
        }
    }

    #[test]
    fn a_fresh_salt_goes_into_every_body() {
        let key = base64(OUR_KEY);

        assert_ne!(encrypt(&key, b"same"), encrypt(&key, b"same"));
    }

    /// What was never encrypted does not open: too short for a header, or a header whose record
    /// size can not hold a record, or a first record that fails its tag.
    #[test]
    fn plain_data_does_not_open() {
        let key = base64(OUR_KEY);

        assert_eq!(decrypt(&key, b""), Err(OpenError::NotEncrypted));
        assert_eq!(decrypt(&key, b"hosts: {}\n"), Err(OpenError::NotEncrypted));

        for plain in [
            &b"-----BEGIN CERTIFICATE-----\nMIIB\n-----END CERTIFICATE-----\n"[..],
            br#"{"email":"a@b.com","name":"John Doe"}"#,
        ] {
            assert!(decrypt(&key, plain).is_err(), "{plain:?}");
        }
    }

    /// A key id that runs past the end of the body leaves no records to read.
    #[test]
    fn a_key_id_longer_than_the_body_is_not_a_body() {
        let mut body = vec![0u8; HEADER_LEN];
        body[SALT_LEN..SALT_LEN + 4].copy_from_slice(&4096u32.to_be_bytes());
        body[HEADER_LEN - 1] = 10;

        assert_eq!(
            decrypt(&base64(OUR_KEY), &body),
            Err(OpenError::NotEncrypted)
        );
    }
}
