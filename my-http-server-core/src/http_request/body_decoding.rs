use std::sync::Arc;

use my_http_utils::encryption::{EncryptedBody, HttpBodyDecryptError, AES128GCM_CONTENT_ENCODING};

use crate::{
    body_encryption_keys::open_request_body, BodyEncryptionKey, BodyEncryptionKeys,
    ContentEncoding, EncryptionInfo, HttpFailResult, HttpOutput, HttpResponseHeaders,
    WebContentType,
};

/// The content codings a request body may come in, as a `415` lists them in `Accept-Encoding`
/// (RFC 9110 §15.5.16). `aes128gcm` is not among them: a server answers `415` for it only when it
/// has no key to open it.
const DECODED_REQUEST_ENCODINGS: &str = "gzip, deflate, br, zstd";

/// What turns the bytes that came over the wire into the body a handler reads, settled from the
/// `Content-Encoding` of the request before a byte of the body is read.
pub(crate) enum BodyDecoding {
    /// No encoding announced: the body is taken as it is.
    None,
    Decompress(ContentEncoding),
    /// `Content-Encoding: aes128gcm`, alone or after a compression (`gzip, aes128gcm`, what
    /// my-http-utils' `compress().encrypt()` sends): decrypted with the keys of the server as they
    /// were when the body was read, tried in turn, then decompressed when it was compressed first.
    Decrypt {
        keys: Arc<Vec<BodyEncryptionKey>>,
        /// [`ContentEncoding::None`], or the compression applied before the encryption.
        then: ContentEncoding,
    },
}

impl BodyDecoding {
    /// An encoding the server can not undo is refused here, before the body is read: an unknown
    /// one with a `400`, and `aes128gcm` with no key to open it with a `415`.
    pub(crate) fn new(
        content_encoding: Option<&str>,
        encryption_keys: Option<&BodyEncryptionKeys>,
    ) -> Result<Self, HttpFailResult> {
        let Some(compressed_first) = content_encoding.and_then(compressed_before_encryption) else {
            return match ContentEncoding::new(content_encoding)? {
                ContentEncoding::None => Ok(Self::None),
                compression => Ok(Self::Decompress(compression)),
            };
        };

        // What was applied before the encryption: nothing, or one compression.
        let then = match ContentEncoding::new(Some(compressed_first)) {
            Ok(ContentEncoding::Aes128Gcm) | Err(_) => {
                return Err(unsupported_content_encoding(
                    content_encoding.unwrap_or_default(),
                ))
            }
            Ok(compression) => compression,
        };

        let keys = encryption_keys
            .map(BodyEncryptionKeys::snapshot)
            .filter(|keys| !keys.is_empty());

        match keys {
            Some(keys) => Ok(Self::Decrypt { keys, then }),
            None => Err(no_key_to_decrypt_the_body()),
        }
    }

    pub(crate) fn is_none(&self) -> bool {
        matches!(self, Self::None)
    }

    /// The body as a handler reads it. Decompressing and decrypting are CPU work, so the caller
    /// runs this on the blocking pool.
    pub(crate) fn decode(
        self,
        body: Vec<u8>,
        max_decompressed_size: usize,
    ) -> Result<DecodedBody, HttpFailResult> {
        match self {
            Self::None => Ok(DecodedBody::plain(body)),
            Self::Decompress(compression) => compression
                .decompress_if_needed(body.into(), max_decompressed_size)
                .map(DecodedBody::plain),
            Self::Decrypt { keys, then } => {
                // The standard has an empty body be empty bytes, both ways: nothing is encrypted
                // in it, so no key opens it and it carries no salt and no clock. It is handed on
                // as an empty body, without encryption info - it proves nothing about who sent
                // it.
                if body.is_empty() {
                    return Ok(DecodedBody::plain(body));
                }

                let (key, opened) = open_request_body(&keys, &body).map_err(body_does_not_open)?;

                let EncryptedBody {
                    salt,
                    timestamp,
                    body: plain,
                } = opened;

                let encryption_info = EncryptionInfo::new(key.clone(), &body, salt, timestamp);

                let bytes = match then {
                    ContentEncoding::None => plain,
                    compression => {
                        compression.decompress_if_needed(plain.into(), max_decompressed_size)?
                    }
                };

                Ok(DecodedBody {
                    bytes,
                    encryption_info: Some(encryption_info),
                })
            }
        }
    }
}

/// `Content-Encoding` lists the codings in the order they were applied (RFC 9110 §8.4), so a body
/// whose list ends with `aes128gcm` was encrypted last, and is decrypted first. `Some` with what
/// the list holds before it (`""` for `aes128gcm` alone), `None` when the body was not encrypted
/// last.
fn compressed_before_encryption(content_encoding: &str) -> Option<&str> {
    let (before, last) = content_encoding
        .rsplit_once(',')
        .unwrap_or(("", content_encoding));

    last.trim()
        .eq_ignore_ascii_case(AES128GCM_CONTENT_ENCODING)
        .then_some(before)
}

/// A body as a handler reads it, and how it came encrypted when it did.
pub(crate) struct DecodedBody {
    pub(crate) bytes: Vec<u8>,
    pub(crate) encryption_info: Option<EncryptionInfo>,
}

impl DecodedBody {
    pub(crate) fn plain(bytes: Vec<u8>) -> Self {
        Self {
            bytes,
            encryption_info: None,
        }
    }
}

/// `400`, as [`ContentEncoding::new`] answers an encoding it does not know: here, a list of
/// codings this server does not undo - two compressions, or `aes128gcm` anywhere but last.
fn unsupported_content_encoding(content_encoding: &str) -> HttpFailResult {
    HttpFailResult::as_validation_error(format!(
        "Unsupported content encoding: {}",
        content_encoding.trim()
    ))
}

/// `415`: the server takes no encrypted bodies, or has no keys yet. RFC 9110 §15.5.16 has the
/// answer to a content coding the server does not support carry `Accept-Encoding` with the ones
/// it does.
fn no_key_to_decrypt_the_body() -> HttpFailResult {
    let mut headers = HttpResponseHeaders::new(Some(WebContentType::Text));
    headers.add_header(
        "Accept-Encoding".into(),
        DECODED_REQUEST_ENCODINGS.to_string(),
    );

    HttpFailResult::new(
        HttpOutput::Content {
            status_code: 415,
            headers,
            content: format!(
                "Unsupported Media Type: the request body is encrypted (Content-Encoding: {}), and this server has no key to decrypt it",
                AES128GCM_CONTENT_ENCODING
            )
            .into_bytes(),
        },
        false,
        true,
    )
}

/// `400`, the way a compressed body that does not decode is answered: the client sent a body the
/// server can not read - no key opens it, it is damaged or cut short, or what it holds is shorter
/// than the sender's clock it has to start with. The text does not say how many keys the server
/// has.
fn body_does_not_open(err: HttpBodyDecryptError) -> HttpFailResult {
    HttpFailResult::as_validation_error(format!(
        "Can not read the encrypted request body (Content-Encoding: {}): {}",
        AES128GCM_CONTENT_ENCODING, err
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn aes128gcm_last_is_split_off_the_list() {
        assert_eq!(compressed_before_encryption("aes128gcm"), Some(""));
        assert_eq!(compressed_before_encryption(" AES128GCM "), Some(""));
        assert_eq!(
            compressed_before_encryption("gzip, aes128gcm"),
            Some("gzip")
        );
        assert_eq!(compressed_before_encryption("gzip,aes128gcm"), Some("gzip"));
        assert_eq!(
            compressed_before_encryption("br, gzip, aes128gcm"),
            Some("br, gzip")
        );

        assert_eq!(compressed_before_encryption("gzip"), None);
        assert_eq!(compressed_before_encryption("aes128gcm, gzip"), None);
        assert_eq!(compressed_before_encryption(""), None);
    }

    fn keys() -> BodyEncryptionKeys {
        BodyEncryptionKeys::new(vec![BodyEncryptionKey::from_bytes("k", &[1; 32]).unwrap()])
    }

    fn decoding(content_encoding: &str) -> Result<BodyDecoding, HttpFailResult> {
        BodyDecoding::new(Some(content_encoding), Some(&keys()))
    }

    #[test]
    fn a_compression_then_the_encryption_is_decrypted_then_decompressed() {
        for (content_encoding, then) in [
            ("aes128gcm", ContentEncoding::None),
            ("identity, aes128gcm", ContentEncoding::None),
            ("gzip, aes128gcm", ContentEncoding::GZip),
            ("br, aes128gcm", ContentEncoding::Br),
            ("zstd, aes128gcm", ContentEncoding::Zstd),
            ("deflate, aes128gcm", ContentEncoding::Deflate),
        ] {
            match decoding(content_encoding) {
                Ok(BodyDecoding::Decrypt { then: got, .. }) => {
                    assert_eq!(got, then, "{content_encoding}")
                }
                _ => panic!("{content_encoding}: expected Decrypt"),
            }
        }
    }

    #[test]
    fn a_list_the_server_does_not_undo_is_a_400() {
        for content_encoding in [
            "aes128gcm, aes128gcm",
            "gzip, br, aes128gcm",
            "aes128gcm, gzip",
            "gzip, br",
            "unknown, aes128gcm",
        ] {
            let Err(err) = decoding(content_encoding) else {
                panic!("{content_encoding}: expected an error");
            };

            assert_eq!(err.output.get_status_code(), 400, "{content_encoding}");
        }
    }

    #[test]
    fn without_keys_an_encrypted_body_is_a_415_compressed_or_not() {
        for content_encoding in ["aes128gcm", "gzip, aes128gcm"] {
            for keys in [None, Some(&BodyEncryptionKeys::default())] {
                let Err(err) = BodyDecoding::new(Some(content_encoding), keys) else {
                    panic!("{content_encoding}: expected an error");
                };

                assert_eq!(err.output.get_status_code(), 415, "{content_encoding}");
            }
        }
    }

    /// The standard: empty bytes are an empty body. Nothing in them is encrypted, so there is no
    /// encryption info to give.
    #[test]
    fn an_empty_body_is_an_empty_body_without_encryption_info() {
        for content_encoding in ["aes128gcm", "gzip, aes128gcm"] {
            let decoded = decoding(content_encoding)
                .unwrap_or_else(|_| panic!("{content_encoding}"))
                .decode(Vec::new(), 1024)
                .unwrap_or_else(|_| panic!("{content_encoding}"));

            assert!(decoded.bytes.is_empty());
            assert!(decoded.encryption_info.is_none());
        }
    }
}
