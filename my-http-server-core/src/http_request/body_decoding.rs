use std::sync::Arc;

use crate::{
    aes128gcm::{self, OpenError},
    body_encryption_keys::decrypt_with_any,
    BodyEncryptionKey, BodyEncryptionKeys, ContentEncoding, EncryptionInfo, HttpFailResult,
    HttpOutput, HttpResponseHeaders, WebContentType,
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
    /// `Content-Encoding: aes128gcm`: the keys of the server as they were when the body was read,
    /// tried in turn.
    Decrypt(Arc<Vec<BodyEncryptionKey>>),
}

impl BodyDecoding {
    /// An encoding the server can not undo is refused here, before the body is read: an unknown
    /// one with a `400`, and `aes128gcm` with no key to open it with a `415`.
    pub(crate) fn new(
        content_encoding: Option<&str>,
        encryption_keys: Option<&BodyEncryptionKeys>,
    ) -> Result<Self, HttpFailResult> {
        match ContentEncoding::new(content_encoding)? {
            ContentEncoding::None => Ok(Self::None),
            ContentEncoding::Aes128Gcm => {
                let keys = encryption_keys
                    .map(BodyEncryptionKeys::snapshot)
                    .filter(|keys| !keys.is_empty());

                match keys {
                    Some(keys) => Ok(Self::Decrypt(keys)),
                    None => Err(no_key_to_decrypt_the_body()),
                }
            }
            compression => Ok(Self::Decompress(compression)),
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
            Self::Decrypt(keys) => {
                let (bytes, key) = decrypt_with_any(&keys, &body).map_err(body_does_not_open)?;

                Ok(DecodedBody {
                    bytes,
                    encryption_info: EncryptionInfo::read(key.clone(), &body),
                })
            }
        }
    }
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
                aes128gcm::CONTENT_ENCODING
            )
            .into_bytes(),
        },
        false,
        true,
    )
}

/// `400`, the way a compressed body that does not decode is answered: the client sent a body the
/// server can not read. The text does not say how many keys the server has.
fn body_does_not_open(err: OpenError) -> HttpFailResult {
    HttpFailResult::as_validation_error(format!(
        "Can not decrypt the request body (Content-Encoding: {}): {}",
        aes128gcm::CONTENT_ENCODING,
        err
    ))
}
