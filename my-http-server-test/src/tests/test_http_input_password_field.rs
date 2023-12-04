use my_http_server::{
    macros::{http_input_field, MyHttpInput},
    HttpFailResult,
};
use rust_extensions::StrOrString;

#[http_input_field("Password")]
pub struct PasswordField(String);

fn process_value(src: &str) -> Result<StrOrString, HttpFailResult> {
    //Password Validation

    if src.len() < 8 {
        return Err(HttpFailResult::as_validation_error(
            "Password must be at least 8 characters long".to_string(),
        ));
    }

    let src = src.trim();
    let src = src.to_lowercase();
    Ok(StrOrString::create_as_string(src))
}

#[cfg(test)]
mod tests {

    use my_http_server::controllers::documentation::DataTypeProvider;

    use super::*;

    #[test]
    fn test() {
        let data_type = PasswordField::get_data_type();
        assert_eq!("SimpleType(Password)", format!("{:?}", data_type));
    }
}
