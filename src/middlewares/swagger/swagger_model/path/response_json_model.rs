use serde::{Deserialize, Serialize};

use crate::middlewares::controllers::documentation::{data_types::HttpDataType, out_results::HttpResult};

#[derive(Serialize, Deserialize, Debug)]
pub struct OutSchemaJsonModel {
    #[serde(rename = "type")]
    #[serde(skip_serializing_if = "Option::is_none")]
    x_type: Option<String>,

    #[serde(rename = "$ref")]
    #[serde(skip_serializing_if = "Option::is_none")]
    x_ref: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ResponseJsonModel {
    #[serde(rename = "x-nullable")]
    nullable: bool,
    description: String,
    schema: OutSchemaJsonModel,
}

impl ResponseJsonModel {
    pub fn new(src: &HttpResult) -> Self {
        Self {
            nullable: src.nullable,
            description: src.description.to_string(),
            schema: get_schema(src),
        }
    }
}

fn get_schema(src: &HttpResult) -> OutSchemaJsonModel {
    match &src.data_type {
        HttpDataType::SimpleType(param_type) => OutSchemaJsonModel {
            x_type: Some(param_type.as_str().to_string()),
            x_ref: None,
        },
        HttpDataType::Object{struct_id} => OutSchemaJsonModel {
            x_type: None,
            x_ref: Some(format!("#/definitions/{}", struct_id)),
        },
        HttpDataType::None => OutSchemaJsonModel {
            x_type: None,
            x_ref: None,
        },
        HttpDataType::ArrayOf(_) => {
            //TODO - Not Implemented yet
            OutSchemaJsonModel {
                x_type: None,
                x_ref: None,
            }
        }
    }
}
