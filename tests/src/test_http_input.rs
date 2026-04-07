use serde::*;


use my_http_server::macros::*;

#[derive(MyHttpInputObjectStructure, Serialize, Deserialize)]
pub struct QueueInterval {
    #[serde(rename = "fromId")]
    pub from_id: String,
    #[serde(rename = "toId")]
    pub to_id: String,
}
