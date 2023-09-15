use crate::controllers::documentation::HttpObjectFields;

use super::yaml_writer::YamlWriter;

pub fn build(yaml_writer: &mut YamlWriter, data: &HttpObjectFields) {
    yaml_writer.write("type", "object");

    yaml_writer.write_array(
        "required",
        data.fields
            .iter()
            .filter(|itm| itm.required)
            .map(|itm| itm.name.as_str().into()),
    );

    write_properties(yaml_writer, data);
}

fn write_properties(yaml_writer: &mut YamlWriter, data: &HttpObjectFields) {
    yaml_writer.write_upper_level("properties", |yaml_writer| {
        for field in &data.fields {
            super::http_data_type::build(yaml_writer, field.name.as_str(), &field.data_type);
        }
    });
}
