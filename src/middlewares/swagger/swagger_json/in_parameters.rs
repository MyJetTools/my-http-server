use crate::middlewares::{
    controllers::documentation::{
        data_types::{EnumType, HttpDataType},
        in_parameters::HttpInputParameter,
        HttpActionDescription,
    },
    swagger::json_object_writer::JsonObjectWriter,
};

pub fn build(action_description: &HttpActionDescription) -> JsonObjectWriter {
    let mut result = JsonObjectWriter::as_array();

    if let Some(in_params) = &action_description.input_params {
        for param in in_params {
            result.write_array_object_element(build_parameter(param));
        }
    }

    result
}

fn build_parameter(param: &HttpInputParameter) -> JsonObjectWriter {
    let mut result = JsonObjectWriter::as_object();

    if let Some(enum_object) = build_enum_field(&param.field.data_type) {
        result.write_object("enum", enum_object);
    }

    result.write_string_value("in", param.source.as_str());
    result.write_string_value("name", param.field.name.as_str());
    result.write_bool_value("x-nullable", !param.field.required);

    if let Some(param_format) = get_param_format(&param.field.data_type) {
        result.write_string_value("format", param_format);
    }

    if let Some(param_type) = get_param_type(&param.field.data_type) {
        result.write_string_value("type", param_type);
    }

    result.write_string_value("description", param.description.as_str());

    if let Some(schema) = super::http_data_type::build(&param.field.data_type) {
        result.write_object("x-schema", schema);
    }

    result
}

fn get_param_format(data_type: &HttpDataType) -> Option<&str> {
    match data_type {
        HttpDataType::SimpleType(param_type) => Some(param_type.as_str()),
        HttpDataType::ObjectId { struct_id: _ } => None,
        HttpDataType::None => None,
        HttpDataType::ArrayOf(_) => None,
        HttpDataType::Object(_) => None,
        HttpDataType::Enum(_) => None,
    }
}

fn get_param_type(data_type: &HttpDataType) -> Option<&str> {
    match data_type {
        HttpDataType::SimpleType(param_type) => Some(param_type.as_swagger_type()),
        HttpDataType::ObjectId { struct_id: _ } => None,
        HttpDataType::None => None,
        HttpDataType::ArrayOf(_) => None,
        HttpDataType::Object(_) => None,
        HttpDataType::Enum(data) => match &data.enum_type {
            EnumType::Integer => Some("integer"),
            EnumType::String => Some("string"),
        },
    }
}

fn build_enum_field(data_type: &HttpDataType) -> Option<JsonObjectWriter> {
    if let HttpDataType::Enum(enum_struct) = data_type {
        let mut result = JsonObjectWriter::as_array();

        for case in &enum_struct.cases {
            result.write_string_element(case.value.as_str());
        }

        return Some(result);
    }

    None
}
