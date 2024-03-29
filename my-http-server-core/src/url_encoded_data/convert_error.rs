use url_utils::url_encoded_data_reader::ReadingEncodedDataError;

use crate::HttpFailResult;

pub fn convert_error<TOk>(
    name: &str,
    result: Result<TOk, ReadingEncodedDataError>,
    data_source: &str,
) -> Result<TOk, HttpFailResult> {
    match result {
        Ok(result) => Ok(result),
        Err(err) => match err {
            ReadingEncodedDataError::RequiredParameterIsMissing(param_name) => {
                return Err(HttpFailResult::required_parameter_is_missing(
                    param_name.as_str(),
                    data_source,
                ));
            }
            ReadingEncodedDataError::CanNotParseValue(value) => {
                return Err(HttpFailResult::invalid_value_to_parse(format!(
                    "Can no parse {} value {} from {}",
                    name, value, data_source
                )));
            }
            ReadingEncodedDataError::UrlDecodeError(err) => {
                return Err(HttpFailResult::as_fatal_error(format!(
                    "Parameter {} has problem with UrlDecodeError in {}. Err: {:?}",
                    name, data_source, err,
                )));
            }
        },
    }
}
