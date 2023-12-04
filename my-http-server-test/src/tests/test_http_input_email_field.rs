use my_http_server::{
    macros::{http_input_field, MyHttpInput},
    HttpFailResult,
};
use rust_extensions::StrOrString;

#[http_input_field]
pub struct EmailField(String);

fn process_value(src: &str) -> Result<StrOrString, HttpFailResult> {
    let src = src.trim();
    //Email Validation
    let src = src.to_lowercase();
    Ok(StrOrString::create_as_string(src))
}

#[cfg(test)]
mod tests {

    use my_http_server::controllers::documentation::DataTypeProvider;

    use super::*;

    #[test]
    fn test() {
        let data_type = EmailField::get_data_type();
        assert_eq!("SimpleType(String)", format!("{:?}", data_type));
    }
}
