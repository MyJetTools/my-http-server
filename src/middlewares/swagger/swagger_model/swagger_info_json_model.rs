use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct SwaggerInfoJsonModel {
    pub title: String,
    pub version: String,
}
