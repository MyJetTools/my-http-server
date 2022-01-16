
use serde::{Deserialize, Serialize};

use crate::middlewares::controllers::documentation::data_types::{HttpDataType, ArrayElement};
#[derive(Serialize, Deserialize, Debug)]
pub struct SwaggerHttpArrayItem{
    #[serde(rename = "type")]
    #[serde(skip_serializing_if = "Option::is_none")]
    x_type: Option<String>,
    #[serde(rename = "$ref")]
    #[serde(skip_serializing_if = "Option::is_none")]
    x_ref: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SwaggerHttpDataType {
    #[serde(rename = "$ref")]
    #[serde(skip_serializing_if = "Option::is_none")]
    x_ref: Option<String>,
    #[serde(rename = "type")]
    #[serde(skip_serializing_if = "Option::is_none")]
    x_type: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    items: Option<SwaggerHttpArrayItem>
}

impl SwaggerHttpDataType {
    pub fn new(data_type: &HttpDataType) -> Option<Self> {
        match data_type {
            HttpDataType::SimpleType(param_type) => Self {
                x_ref: None,
                x_type: Some(param_type.as_str().to_string()),
                items: None
            }
            .into(),
            HttpDataType::Object{struct_id} => Self {
                x_ref: Some(format!("#/definitions/{}",struct_id)),
                x_type: None,
                items: None
            }
            .into(),
            HttpDataType::None => None,
            HttpDataType::ArrayOf(array_element) => {

               let items = match array_element{
                    ArrayElement::SimpleType(param_type) => 
                         SwaggerHttpArrayItem{ x_type: Some(param_type.as_str().to_string()), x_ref: None }
                        ,
                    ArrayElement::Object { struct_id } => SwaggerHttpArrayItem{ x_type: None, x_ref: Some(format!("#/definitions/{}",struct_id)) },
                };

                Self {
                    x_ref: None,
                    x_type: Some("array".to_string()),
                    items: Some(items)
                }
                .into()
 
            }
        }
    }
}
