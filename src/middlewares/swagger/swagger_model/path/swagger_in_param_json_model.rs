use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::middlewares::controllers::documentation::{types::HttpDataType, HttpInputParameter};

#[derive(Serialize, Deserialize, Debug)]
pub struct SwaggerInParamJsonModel {
    #[serde(rename = "type")]
    #[serde(skip_serializing_if = "Option::is_none")]
    p_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    schema: Option<HashMap<String, String>>,
    name: String,
    #[serde(rename = "in")]
    p_in: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    format: Option<String>,
    #[serde(rename = "x-nullable")]
    nullable: bool,
    description: String,
}

impl Into<SwaggerInParamJsonModel> for HttpInputParameter {
    fn into(self) -> SwaggerInParamJsonModel {
        SwaggerInParamJsonModel {
            p_in: self.source.as_str().to_string(),
            name: self.name,
            format: get_param_format(&self.data_type),
            nullable: !self.required,
            p_type: get_param_type(&self.data_type),
            description: self.description,
            schema: get_schema(&self.data_type),
        }
    }
}

fn get_schema(data_type: &HttpDataType) -> Option<HashMap<String, String>> {
    match data_type {
        HttpDataType::SimpleType(_) => None,
        HttpDataType::Object(object_description) => {
            let mut data = HashMap::new();
            data.insert(
                "$ref".to_string(),
                format!("#/definitions/{}", object_description.struct_id),
            );

            Some(data)
        }
        HttpDataType::None => None,
    }
}

fn get_param_format(data_type: &HttpDataType) -> Option<String> {
    match data_type {
        HttpDataType::SimpleType(param_type) => Some(param_type.as_str().to_string()),
        HttpDataType::Object(_) => None,
        HttpDataType::None => None,
    }
}

fn get_param_type(data_type: &HttpDataType) -> Option<String> {
    match data_type {
        HttpDataType::SimpleType(param_type) => Some(param_type.as_swagger_type().to_string()),
        HttpDataType::Object(_) => None,
        HttpDataType::None => None,
    }
}
