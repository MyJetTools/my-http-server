use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::middlewares::controllers::documentation::{
    types::{ArrayElement, HttpDataType},
    HttpInputParameter,
};
#[derive(Serialize, Deserialize, Debug)]
pub struct InParamSchemaItems {
    #[serde(rename = "type")]
    #[serde(skip_serializing_if = "Option::is_none")]
    x_type: Option<String>,
    #[serde(rename = "ref")]
    #[serde(skip_serializing_if = "Option::is_none")]
    x_ref: Option<HashMap<String, String>>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct InParamSchema {
    #[serde(rename = "$ref")]
    #[serde(skip_serializing_if = "Option::is_none")]
    x_ref: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    items: Option<InParamSchemaItems>,

    #[serde(rename = "type")]
    #[serde(skip_serializing_if = "Option::is_none")]
    x_type: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SwaggerInParamJsonModel {
    #[serde(rename = "type")]
    #[serde(skip_serializing_if = "Option::is_none")]
    p_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    schema: Option<InParamSchema>,
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

fn get_schema(data_type: &HttpDataType) -> Option<InParamSchema> {
    match data_type {
        HttpDataType::SimpleType(_) => None,
        HttpDataType::Object(object_description) => Some(InParamSchema {
            x_ref: Some(format!("#/definitions/{}", object_description.struct_id)),

            x_type: None,
            items: None,
        }),
        HttpDataType::None => None,
        HttpDataType::ArrayOf(array_element) => match array_element {
            ArrayElement::SimpleType(param_type) => {
                let items = InParamSchemaItems {
                    x_type: Some(param_type.as_swagger_type().to_string()),
                    x_ref: None,
                };

                let result = InParamSchema {
                    x_ref: None,
                    x_type: Some("array".to_string()),
                    items: Some(items),
                };

                Some(result)
            }

            ArrayElement::Object(object_description) => {
                let mut x_ref = HashMap::new();
                x_ref.insert(
                    "$ref".to_string(),
                    format!("#/definitions/{}", object_description.struct_id),
                );

                let items = InParamSchemaItems {
                    x_type: None,
                    x_ref: Some(x_ref),
                };

                let schema = InParamSchema {
                    x_ref: None,
                    items: Some(items),
                    x_type: Some("array".to_string()),
                };

                Some(schema)
            }
        },
    }
}

fn get_param_format(data_type: &HttpDataType) -> Option<String> {
    match data_type {
        HttpDataType::SimpleType(param_type) => Some(param_type.as_str().to_string()),
        HttpDataType::Object(_) => None,
        HttpDataType::None => None,
        HttpDataType::ArrayOf(_) => None,
    }
}

fn get_param_type(data_type: &HttpDataType) -> Option<String> {
    match data_type {
        HttpDataType::SimpleType(param_type) => Some(param_type.as_swagger_type().to_string()),
        HttpDataType::Object(_) => None,
        HttpDataType::None => None,
        HttpDataType::ArrayOf(_) => None,
    }
}
