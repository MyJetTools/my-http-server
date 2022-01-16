use crate::middlewares::{
    controllers::documentation::data_types::{ArrayElement, HttpDataType, HttpSimpleType},
    swagger::json_object_writer::JsonObjectWriter,
};

pub fn build(data_type: &HttpDataType) -> Option<JsonObjectWriter> {
    match data_type {
        HttpDataType::SimpleType(param_type) => Some(build_simple_type(param_type)),
        HttpDataType::ObjectId { struct_id } => Some(build_object_type(struct_id)),
        HttpDataType::Object(object_type) => Some(build_object_type(&object_type.struct_id)),
        HttpDataType::None => None,
        HttpDataType::ArrayOf(array_element) => {
            let items = match array_element {
                ArrayElement::SimpleType(param_type) => build_simple_type(param_type),
                ArrayElement::ObjectId { struct_id } => build_object_type(struct_id),
                ArrayElement::Object(object_type) => build_object_type(&object_type.struct_id),
            };

            let mut result = JsonObjectWriter::as_object();
            result.write_string_value("type", "array");
            result.write_object("items", items);
            Some(result)
        }
    }
}

fn build_simple_type(param_type: &HttpSimpleType) -> JsonObjectWriter {
    let mut result = JsonObjectWriter::as_object();
    result.write_string_value("type", param_type.as_str());
    result
}

fn build_object_type(struct_id: &str) -> JsonObjectWriter {
    let mut result = JsonObjectWriter::as_object();
    result.write_string_value("$ref", format!("#/definitions/{}", struct_id).as_str());
    result
}
