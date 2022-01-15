use serde::{Deserialize, Serialize};

use crate::middlewares::controllers::documentation::types::HttpDataType;

#[derive(Serialize, Deserialize, Debug)]
pub struct SwaggerDefinitionProperty {
    #[serde(rename = "$ref")]
    #[serde(skip_serializing_if = "Option::is_none")]
    x_ref: Option<String>,
    #[serde(rename = "type")]
    #[serde(skip_serializing_if = "Option::is_none")]
    x_type: Option<String>,
}

impl SwaggerDefinitionProperty {
    pub fn new(data_type: &HttpDataType) -> Option<Self> {
        match data_type {
            HttpDataType::SimpleType(param_type) => Self {
                x_ref: None,
                x_type: Some(param_type.as_str().to_string()),
            }
            .into(),
            HttpDataType::Object(object_description) => Self {
                x_ref: Some(object_description.struct_id.to_string()),
                x_type: None,
            }
            .into(),
            HttpDataType::None => None,
            HttpDataType::ArrayOf(_) => {
                //TODO - Not Implemented yet
                None
            }
        }
    }
}
