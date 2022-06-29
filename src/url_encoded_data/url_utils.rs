use std::collections::HashMap;

use crate::url_decoder::UrlDecodeError;

use super::UrlEncodedValueAsString;

pub fn parse_query_string<'s>(
    query_string: &'s str,
) -> Result<HashMap<String, UrlEncodedValueAsString<'s>>, UrlDecodeError> {
    let mut result = HashMap::new();
    let elements = query_string.split("&");

    for el in elements {
        let kv = el.find('=');

        if let Some(index) = kv {
            let key = crate::url_decoder::decode_from_url_query_string(&el[..index])?;
            let value = UrlEncodedValueAsString::new(&el[index + 1..]);
            result.insert(key, value);
        }
    }

    Ok(result)
}
