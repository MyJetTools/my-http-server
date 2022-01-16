use crate::middlewares::swagger::json_object_writer::JsonObjectWriter;

pub fn get(host: &str) -> JsonObjectWriter {
    let mut result = JsonObjectWriter::as_array();
    result.write_string_element(host);
    result
}
