use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct SwaggerDefinitionModel {
    #[serde(rename = "type")]
    pub x_type: String,
    pub required: Vec<String>,
}
