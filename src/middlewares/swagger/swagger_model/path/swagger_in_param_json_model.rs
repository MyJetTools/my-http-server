use serde::{Deserialize, Serialize};

use crate::middlewares::controllers::documentation::HttpInputParameter;

#[derive(Serialize, Deserialize, Debug)]
pub struct SwaggerInParamJsonModel {
    #[serde(rename = "type")]
    p_type: String,
    name: String,
    #[serde(rename = "in")]
    p_in: String,
    format: String,
    #[serde(rename = "x-nullable")]
    nullable: bool,
    description: String,
}

impl Into<SwaggerInParamJsonModel> for HttpInputParameter {
    fn into(self) -> SwaggerInParamJsonModel {
        SwaggerInParamJsonModel {
            name: self.name,
            format: self.param_type.as_str().to_string(),
            nullable: !self.required,
            p_in: self.source.to_str().to_string(),
            p_type: self.param_type.as_swagger_type().to_string(),
            description: self.description,
        }
    }
}
