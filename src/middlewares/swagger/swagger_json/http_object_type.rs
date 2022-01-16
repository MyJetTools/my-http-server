use crate::middlewares::{
    controllers::documentation::data_types::HttpObjectType,
    swagger::json_object_writer::JsonObjectWriter,
};

pub fn build(http_object: &HttpObjectType) -> JsonObjectWriter {
    let mut result = JsonObjectWriter::as_object();

    result.write_string_value("key", "object");
    result.write_object("required", compile_required(http_object));
    result.write_object("properties", compile_properties(http_object));

    result
}

fn compile_required(src: &HttpObjectType) -> JsonObjectWriter {
    let mut result = JsonObjectWriter::as_array();

    for prop in &src.properties {
        if prop.required {
            result.write_string_element(prop.name.as_str());
        }
    }

    result
}

fn compile_properties(src: &HttpObjectType) -> JsonObjectWriter {
    let mut result = JsonObjectWriter::as_object();

    for prop in &src.properties {
        if let Some(json_object) = super::http_data_type::build(&prop.data_type) {
            result.write_object(prop.name.as_str(), json_object);
        }
    }

    result
}
