use my_http_server::macros::*;

#[derive(MyHttpInput)]
pub struct ToLowerCaseFromQuery {
    #[http_query(name = "testToLowerString", description = "a", to_lowercase)]
    pub test_to_lower_string: String,

    #[http_query(name = "testToLowerStringOpt", description = "a", to_uppercase)]
    pub test_to_lower_string_opt: Option<String>,
}

#[derive(MyHttpInput)]
pub struct ToLowerCaseFromHeader {
    #[http_header(name = "testToLowerString", description = "a", to_lowercase)]
    pub test_to_lower_string: String,

    #[http_header(name = "testToLowerStringOpt", description = "a", to_uppercase)]
    pub test_to_lower_string_opt: Option<String>,
}

#[derive(MyHttpInput)]
pub struct DefaultQueryInputModel {
    #[http_query(name : "noDefault", description= "Test", to_lowercase)]
    pub no_default: String,

    #[http_query(name = "withDefault"; description: "a", default: "MyDefault", to_lowercase)]
    pub with_default: String,

    #[http_query(name : "noDefaultOpt", description: "Test", to_lowercase)]
    pub no_default_opt: Option<String>,

    #[http_query(name = "withDefaultOpt"; description: "a", default: "MyDefault", to_lowercase)]
    pub with_default_opt: Option<String>,
}

#[derive(MyHttpInput)]
pub struct DefaultQueryInputModelFromHeader {
    #[http_header(name : "noDefault", description: "Test", to_lowercase)]
    pub no_default: String,

    #[http_header(name = "withDefault"; description: "a", default: "MyDefault", to_lowercase)]
    pub with_default: String,

    #[http_header(name : "noDefaultOpt", description: "Test", to_lowercase)]
    pub no_default_opt: Option<String>,

    #[http_header(name = "withDefaultOpt"; description: "a", default: "MyDefault", to_lowercase)]
    pub with_default_opt: Option<String>,
}

#[derive(MyHttpInput)]
pub struct ToLowerCaseFromPath {
    #[http_path(name = "testToLowerString", description = "a", to_lowercase)]
    pub test_to_lower_string: String,
}

#[derive(MyHttpInput)]
pub struct DefaultQueryInputModelFromPath {
    #[http_path(name : "noDefault", description: "Test", to_lowercase)]
    pub no_default: String,

    #[http_path(name = "withDefault"; description: "a", default: "MyDefault", to_lowercase)]
    pub with_default: String,
}

#[derive(MyHttpInput)]
pub struct ToLowerCaseFromBody {
    #[http_body(name = "testToLowerString", description = "a", to_lowercase)]
    pub test_to_lower_string: String,

    #[http_body(name = "testToLowerStringOpt", description = "a", to_uppercase)]
    pub test_to_lower_string_opt: Option<String>,
}

#[derive(MyHttpInput)]
pub struct DefaultQueryInputModelFromBody {
    #[http_body(name : "noDefault", description: "Test", to_lowercase)]
    pub no_default: String,

    #[http_body(name = "withDefault"; description: "a", default: "MyDefault", to_lowercase)]
    pub with_default: String,

    #[http_body(name : "noDefaultOpt", description: "Test", to_lowercase)]
    pub no_default_opt: Option<String>,

    #[http_body(name = "withDefaultOpt"; description: "a", default: "MyDefault", to_lowercase)]
    pub with_default_opt: Option<String>,
}
