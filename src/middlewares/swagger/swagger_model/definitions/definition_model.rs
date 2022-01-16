use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::middlewares::controllers::documentation::data_types::HttpObjectType;

use super::swagger_http_data_type::SwaggerHttpDataType;


#[derive(Serialize, Deserialize, Debug)]
pub struct SwaggerDefinitionModel {
    #[serde(rename = "type")]
    pub x_type: String,
    pub required: Vec<String>,
    pub properties: BTreeMap<String, SwaggerHttpDataType>,
}

impl SwaggerDefinitionModel {
    pub fn from_object(src: &HttpObjectType) -> Self {
        Self {
            x_type: "object".to_string(),
            required: compile_required(src),
            properties: compile_properties(src),
        }
    }
}

fn compile_required(src: &HttpObjectType) -> Vec<String> {
    let mut result = Vec::new();

    for prop in &src.properties {
        if prop.required {
            result.push(prop.name.to_string());
        }
    }

    result
}

fn compile_properties(src: &HttpObjectType) -> BTreeMap<String, SwaggerHttpDataType> {
    let mut result = BTreeMap::new();

    for prop in &src.properties {
        if let Some(swagger) = SwaggerHttpDataType::new(&prop.data_type) {
            result.insert(prop.name.to_string(), swagger);
        }
    }

    result
}
