use my_http_server::{
    macros::{http_input_field, MyHttpInput},
    HttpFailResult,
};
use rust_extensions::StrOrString;

#[http_input_field(open_api_type:"Password")]
pub struct PasswordField(String);

fn process_value(src: &str) -> Result<StrOrString, HttpFailResult> {
    if src.as_bytes().get(0).unwrap() <= &32u8 {
        return Err(HttpFailResult::as_validation_error(
            "Password must not be started".to_string(),
        ));
    }
    //Email Validation

    if src.len() < 8 {
        return Err(HttpFailResult::as_validation_error(
            "Password must be at least 8 characters long".to_string(),
        ));
    }

    let src = src.trim();
    let src = src.to_lowercase();
    Ok(StrOrString::create_as_string(src))
}
