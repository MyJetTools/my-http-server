use super::{UrlDecodeError, UrlDecoder};

pub fn decode_from_url_query_string(src: &str) -> Result<String, UrlDecodeError> {
    let index_percent = src.find("%");

    if index_percent.is_none() {
        return Ok(src.to_string());
    }

    let mut result: Vec<u8> = Vec::with_capacity(src.len());
    let mut url_decoder = UrlDecoder::new(src);

    while let Some(next_one) = url_decoder.get_next()? {
        result.push(next_one);
    }

    return Ok(String::from_utf8(result).unwrap());
}

#[cfg(test)]
mod tests {

    #[test]
    fn test_url_decoding() {
        let value = "http%3A%2F%2F127.0.0.1%3A5223";

        let result = super::decode_from_url_query_string(value);

        assert_eq!("http://127.0.0.1:5223", result.unwrap().as_str());
    }

    #[test]
    fn test_url_decoding_case_2() {
        let value = "Euro%20Stoxx%2050";

        let result = super::decode_from_url_query_string(value);

        assert_eq!("Euro Stoxx 50", result.unwrap().as_str());
    }
}
