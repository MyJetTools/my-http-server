use my_http_server::macros::*;
use my_http_server::*;
#[derive(MyHttpInput, Debug)]
pub struct TestPathModel {
    #[http_path(name = "pathKey", description = "Path key")]
    pub path_key: String,

    #[http_query(name = "queryData", description = "DataFromQuery")]
    pub query_data: String,
}
