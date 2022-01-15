use serde::{Deserialize, Serialize};

use crate::middlewares::controllers::documentation::{types::HttpDataType, HttpInputParameter};

#[derive(Serialize, Deserialize, Debug)]
pub struct SwaggerInParamJsonModel {
    #[serde(rename = "type")]
    #[serde(skip_serializing_if = "Option::is_none")]
    p_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    scheme: Option<String>,
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
        match self {
            HttpInputParameter::Path(data) => SwaggerInParamJsonModel {
                p_in: "path".to_string(),
                name: data.name,
                format: get_param_format(&data.data_type),
                nullable: !data.required,
                p_type: get_param_type(&data.data_type),
                description: data.description,
                scheme: get_scheme(&data.data_type),
            },
            HttpInputParameter::Query(data) => SwaggerInParamJsonModel {
                p_in: "query".to_string(),
                name: data.name,
                format: get_param_format(&data.data_type),
                nullable: !data.required,
                p_type: get_param_type(&data.data_type),
                description: data.description,
                scheme: get_scheme(&data.data_type),
            },
            HttpInputParameter::Header(data) => SwaggerInParamJsonModel {
                p_in: "header".to_string(),
                name: data.name,
                format: get_param_format(&data.data_type),
                nullable: !data.required,
                p_type: get_param_type(&data.data_type),
                description: data.description,
                scheme: get_scheme(&data.data_type),
            },
            HttpInputParameter::FormData(data) => SwaggerInParamJsonModel {
                p_in: "formData".to_string(),
                name: data.name,
                format: get_param_format(&data.data_type),
                nullable: !data.required,
                p_type: get_param_type(&data.data_type),
                description: data.description,
                scheme: get_scheme(&data.data_type),
            },
            HttpInputParameter::Body(data) => SwaggerInParamJsonModel {
                p_in: "body".to_string(),
                name: data.name,
                format: get_param_format(&data.data_type),
                nullable: !data.required,
                p_type: get_param_type(&data.data_type),
                description: data.description,
                scheme: get_scheme(&data.data_type),
            },
        }
    }
}

fn get_scheme(data_type: &HttpDataType) -> Option<String> {
    match data_type {
        HttpDataType::SimpleType {
            required: _,
            param_type: _,
        } => None,
        HttpDataType::Object {
            required: _,
            object_description,
        } => Some(format!("#/definitions/{}", object_description.struct_id)),
        HttpDataType::None => None,
    }
}

fn get_param_format(data_type: &HttpDataType) -> Option<String> {
    match data_type {
        HttpDataType::SimpleType {
            required: _,
            param_type,
        } => Some(param_type.as_str().to_string()),
        HttpDataType::Object {
            required: _,
            object_description: _,
        } => None,
        HttpDataType::None => None,
    }
}

fn get_param_type(data_type: &HttpDataType) -> Option<String> {
    match data_type {
        HttpDataType::SimpleType {
            required: _,
            param_type,
        } => Some(param_type.as_swagger_type().to_string()),
        HttpDataType::Object {
            required: _,
            object_description: _,
        } => None,
        HttpDataType::None => None,
    }
}
