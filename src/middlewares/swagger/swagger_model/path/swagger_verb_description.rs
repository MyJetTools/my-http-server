use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::{
    middlewares::controllers::documentation::{
        types::HttpDataType, HttpActionDescription, HttpInputParameter, HttpResult,
    },
    WebContentType,
};

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
    pub fn new(action_description: HttpActionDescription) -> Self {
        let produces = compile_produces_field(&action_description);

        Self {
            tags: vec![action_description.name.to_string()],
            description: action_description.description.to_string(),
            parameters: into_json_parameters(action_description.input_params),
            produces,
            responses: compile_responses(action_description.results.as_slice()),
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

pub fn compile_produces_field(action_description: &HttpActionDescription) -> Vec<String> {
    let mut result = Vec::new();

    for http_result in &action_description.results {
        let produce_type = match http_result.data_type {
            HttpDataType::SimpleType(_) => Some(WebContentType::Text.as_str()),
            HttpDataType::Object(_) => Some(WebContentType::Json.as_str()),
            HttpDataType::None => None,
            HttpDataType::ArrayOf(_) => None,
        };

        if let Some(produce_type) = produce_type {
            if !result.iter().any(|itm| itm == produce_type) {
                result.push(produce_type.to_string());
            }
        }
    }

    result
}

fn compile_responses(results: &[HttpResult]) -> HashMap<String, ResponseJsonModel> {
    let mut result = HashMap::new();

    for http_result in results {
        result.insert(
            format!("{}", http_result.http_code),
            ResponseJsonModel::new(http_result),
        );
    }

    result
}
