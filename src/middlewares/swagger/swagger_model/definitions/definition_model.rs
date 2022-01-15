use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::middlewares::controllers::documentation::types::HttpObjectDescription;

use super::SwaggerDefinitionProperty;

#[derive(Serialize, Deserialize, Debug)]
pub struct SwaggerDefinitionModel {
    #[serde(rename = "type")]
    pub x_type: String,
    pub required: Vec<String>,
    pub properties: BTreeMap<String, SwaggerDefinitionProperty>,
}

impl SwaggerDefinitionModel {
    pub fn from_object(src: &HttpObjectDescription) -> Self {
        Self {
            x_type: "object".to_string(),
            required: compile_required(src),
            properties: compile_properties(src),
        }
    }
}

fn compile_required(src: &HttpObjectDescription) -> Vec<String> {
    let mut result = Vec::new();

    for prop in &src.properties {
        if prop.prop_type.is_required() {
            result.push(prop.name.to_string());
        }
    }

    result
}

fn compile_properties(src: &HttpObjectDescription) -> BTreeMap<String, SwaggerDefinitionProperty> {
    let mut result = BTreeMap::new();

    for prop in &src.properties {
        if let Some(swagger) = SwaggerDefinitionProperty::new(&prop.prop_type) {
            result.insert(prop.name.to_string(), swagger);
        }
    }

    result
}
