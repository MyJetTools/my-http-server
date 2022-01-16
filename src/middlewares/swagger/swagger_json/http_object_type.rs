use crate::middlewares::{
    controllers::documentation::data_types::HttpObjectType,
    swagger::json_object_writer::JsonObjectWriter,
};

pub fn build(http_object: &HttpObjectType) -> JsonObjectWriter {
    let mut result = JsonObjectWriter::as_object();

    result.write_string_value("type", "object");
    result.write_object("required", compile_required(http_object));
    result.write_object("properties", compile_properties(http_object));

    result
}

fn compile_required(src: &HttpObjectType) -> JsonObjectWriter {
    let mut result = JsonObjectWriter::as_array();

    for field in &src.fields {
        if field.required {
            result.write_string_element(field.name.as_str());
        }
    }

    result
}

fn compile_properties(src: &HttpObjectType) -> JsonObjectWriter {
    let mut result = JsonObjectWriter::as_object();

    for field in &src.fields {
        if let Some(json_object) = super::http_data_type::build(&field.data_type) {
            result.write_object(field.name.as_str(), json_object);
        }
    }

    result
}
