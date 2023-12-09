use my_http_server::macros::MyHttpInput;

#[derive(MyHttpInput)]
pub struct ToLowerCaseWithTrimFromQuery {
    #[http_query(name = "testToLowerString", description = "a", to_lowercase, trim)]
    pub test_to_lower_string: String,

    #[http_query(name = "testToLowerStringOpt", description = "a", to_uppercase, trim)]
    pub test_to_lower_string_opt: Option<String>,

    #[http_query(name = "testTrim", description = "a", trim)]
    pub test_trim: String,

    #[http_query(name = "testTrimOpt", description = "a", trim)]
    pub test_opt_trim: Option<String>,
}
