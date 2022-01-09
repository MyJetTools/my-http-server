use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct OutSchemaJsonModel {
    #[serde(rename = "type")]
    x_type: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ResponseJsonModel {
    #[serde(rename = "x-nullable")]
    x_nullable: bool,
    description: String,
    schema: OutSchemaJsonModel,
}

impl ResponseJsonModel {
    pub fn create_default() -> Self {
        ResponseJsonModel {
            x_nullable: false,
            description: "".to_string(),
            schema: OutSchemaJsonModel {
                x_type: "object".to_string(),
            },
        }
    }
}
