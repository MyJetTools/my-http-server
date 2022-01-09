use serde::{Deserialize, Serialize};

use super::SwaggerVerbDescription;

#[derive(Serialize, Deserialize, Debug)]
pub struct SwaggerPathJsonModel {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub get: Option<SwaggerVerbDescription>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub post: Option<SwaggerVerbDescription>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub put: Option<SwaggerVerbDescription>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub delete: Option<SwaggerVerbDescription>,
}

impl SwaggerPathJsonModel {
    pub fn new() -> Self {
        Self {
            get: None,
            post: None,
            put: None,
            delete: None,
        }
    }
}
