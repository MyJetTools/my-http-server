use crate::middlewares::swagger::json_object_writer::JsonObjectWriter;

pub fn write(json_object_writer: &mut JsonObjectWriter, host: &str, title: &str, version: &str) {
    json_object_writer.write_string_value("x-generator", "My-Http-Server-Generator");
    json_object_writer.write_string_value("swagger", "2.0");
    json_object_writer.write_string_value("host", host);

    let mut info_object = JsonObjectWriter::as_object();
    info_object.write_string_value("title", title);
    info_object.write_string_value("version", version);

    json_object_writer.write_object("info", info_object);
}
