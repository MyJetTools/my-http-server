use my_http_server_swagger::{MyHttpInput, MyHttpObjectStructure};
use serde::{Deserialize, Serialize};

#[derive(MyHttpInput)]
pub struct GetKeysInputModel {
    #[http_body(description = "Keys we do request by")]
    pub keys: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug, MyHttpObjectStructure)]
pub struct GetKeyValuesResponse {
    pub data: Vec<KeyValueHttpModel>,
}

#[derive(Serialize, Deserialize, Debug, MyHttpObjectStructure)]
pub struct KeyValueHttpModel {
    pub key: String,
    pub value: String,
}

#[derive(MyHttpInput)]
pub struct DeleteKeysInputModel {
    #[http_body(description = "Keys delete")]
    pub keys: Vec<String>,
}

#[derive(MyHttpInput)]
pub struct PostKeysInputModel {
    #[http_body(description = "Key Values we post")]
    pub values: Vec<KeyValueHttpModel>,
}
