use serde::{Deserialize, Serialize};

use crate::middlewares::controllers::documentation::types::HttpObjectDescription;

#[derive(Serialize, Deserialize, Debug)]
pub struct SwaggerDefinitionModel {
    #[serde(rename = "type")]
    pub x_type: String,
    pub required: Vec<String>,
}

impl SwaggerDefinitionModel {
    pub fn from_object(src: &HttpObjectDescription) -> Self {
        Self {
            x_type: "object".to_string(),
            required: compile_required(src),
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
