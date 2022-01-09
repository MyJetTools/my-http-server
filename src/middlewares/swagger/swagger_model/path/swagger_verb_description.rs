use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::middlewares::controllers::documentation::{HttpActionDescription, HttpInputParameter};

use super::{ResponseJsonModel, SwaggerInParamJsonModel};

#[derive(Serialize, Deserialize, Debug)]
pub struct SwaggerVerbDescription {
    tags: Vec<String>,
    description: String,
    parameters: Vec<SwaggerInParamJsonModel>,
    produces: Vec<String>,
    responses: HashMap<String, ResponseJsonModel>,
}

impl SwaggerVerbDescription {
    pub fn new(
        action_description: HttpActionDescription,
        in_parameters: Option<Vec<HttpInputParameter>>,
    ) -> Self {
        Self {
            tags: vec![action_description.name.to_string()],
            description: action_description.description.to_string(),
            parameters: into_json_parameters(in_parameters),
            produces: vec![action_description.out_content_type.to_string().to_string()],
            responses: create_default_responses(),
        }
    }
}

fn into_json_parameters(src: Option<Vec<HttpInputParameter>>) -> Vec<SwaggerInParamJsonModel> {
    match src {
        Some(src) => {
            let mut result: Vec<SwaggerInParamJsonModel> = Vec::with_capacity(src.len());
            for param in src {
                result.push(param.into());
            }
            result
        }
        None => vec![],
    }
}

fn create_default_responses() -> HashMap<String, ResponseJsonModel> {
    let mut result = HashMap::new();

    result.insert("200".to_string(), ResponseJsonModel::create_default());

    result
}
