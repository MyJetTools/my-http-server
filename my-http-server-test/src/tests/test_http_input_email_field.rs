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
