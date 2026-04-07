use serde::*;


use my_http_server::macros::*;


    #[derive(MyHttpObjectStructure, Serialize)]
        pub struct AwaitDeliveryHttpResponse {
            pub topic_id: String,
            pub queue_id: String,
            pub confirmation_id: i64,
            pub messages: Vec<MessageToDeliverHttpContract>,
    }



#[derive(Serialize, Deserialize, Debug, MyHttpObjectStructure)]
pub struct MessageToDeliverHttpContract {
    pub id: i64,
    pub attempt_no: i32,
    pub headers: Vec<MessageKeyValueJsonModel>,
    pub content: String,
}

#[derive(Serialize, Deserialize, Debug, MyHttpObjectStructure)]
pub struct MessageKeyValueJsonModel {
    pub key: String,
    pub value: String,
}



