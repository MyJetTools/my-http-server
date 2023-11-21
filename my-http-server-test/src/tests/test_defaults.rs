use my_http_server::macros::*;

#[derive(MyHttpInput)]
pub struct DefaultStringQueryInputModel {
    #[http_query(name : "noDefault", description=: "Test")]
    pub no_default: String,

    #[http_query(name = "withDefault"; description: "a", default: "MyDefault")]
    pub with_default: String,

    #[http_query(name : "noDefaultOpt", description=: "Test")]
    pub no_default_opt: Option<String>,

    #[http_query(name = "withDefaultOpt"; description: "a", default: "MyDefault")]
    pub with_default_opt: Option<String>,
}

#[derive(MyHttpInput)]
pub struct DefaultI32QueryInputModel {
    #[http_query(name : "noDefault", description=: "Test")]
    pub no_default: i32,

    #[http_query(name = "withDefault"; description: "a", default: 15)]
    pub with_default: i32,

    #[http_query(name : "noDefaultOpt", description=: "Test")]
    pub no_default_opt: Option<i32>,

    #[http_query(name = "withDefaultOpt"; description: "a", default: 16)]
    pub with_default_opt: Option<i32>,
}
