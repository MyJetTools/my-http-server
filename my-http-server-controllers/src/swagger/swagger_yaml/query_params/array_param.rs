use crate::{
    controllers::documentation::{HttpEnumStructure, HttpSimpleType},
    swagger::swagger_yaml::yaml_writer::YamlWriter,
};

pub fn write_array_input_param(yaml_writer: &mut YamlWriter, simple_type: &HttpSimpleType) {
    yaml_writer.write("type", "array");
    yaml_writer.write_upper_level("items", |yaml_writer| {
        yaml_writer.write("type", simple_type.as_swagger_type())
    });
}

pub fn write_array_enum_case(yaml_writer: &mut YamlWriter, enum_structure: &HttpEnumStructure) {
    yaml_writer.write("type", "array");
    yaml_writer.write_upper_level("items", |yaml_writer| {
        yaml_writer.write_upper_level("schema", |yaml_writer| {
            super::super::object::write_reference_to_object(yaml_writer, enum_structure);
        })
    });
}
