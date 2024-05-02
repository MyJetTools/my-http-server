use crate::controllers::documentation::data_types::{ArrayElement, HttpDataType, HttpSimpleType};

use super::yaml_writer::YamlWriter;

pub fn build(yaml_writer: &mut YamlWriter, root_name: &str, data_type: &HttpDataType) {
    match &data_type {
        HttpDataType::SimpleType(param_type) => {
            yaml_writer.write_upper_level(root_name, |yaml_writer| {
                write_simple_type(yaml_writer, param_type);
            });
        }

        HttpDataType::Object(object_type) => {
            yaml_writer.write_upper_level(root_name, |yaml_writer| {
                super::object::write_reference_to_object(yaml_writer, object_type);
            });
        }
        HttpDataType::Enum(enum_type) => match enum_type.enum_type {
            crate::controllers::documentation::data_types::EnumType::Integer => {
                yaml_writer.write_upper_level(root_name, |yaml_writer| {
                    write_simple_type(yaml_writer, &HttpSimpleType::Integer);
                });
            }
            crate::controllers::documentation::data_types::EnumType::String => {
                yaml_writer.write_upper_level(root_name, |yaml_writer| {
                    write_simple_type(yaml_writer, &HttpSimpleType::String);
                });
            }
        },
        HttpDataType::None => {}
        HttpDataType::ArrayOf(array_element) => {
            yaml_writer.write_upper_level(root_name, |yaml_writer| {
                write_array_element(yaml_writer, array_element);
            });
        }
        HttpDataType::DictionaryOf(array_element) => {
            yaml_writer.write_upper_level(root_name, |yaml_writer| {
                yaml_writer.write("type", "object");

                yaml_writer.write_upper_level("additionalProperties", |yaml_writer| {
                    match array_element {
                        ArrayElement::SimpleType(param_type) => {
                            write_simple_type(yaml_writer, param_type);
                        }
                        ArrayElement::Object(object_type) => {
                            super::object::write_reference_to_object(yaml_writer, object_type);
                        }
                        ArrayElement::Enum(enum_type) => {
                            super::object::write_reference_to_object(yaml_writer, enum_type);
                        }
                    };
                });
            });
        }
        HttpDataType::DictionaryOfArray(array_element) => {
            yaml_writer.write_upper_level(root_name, |yaml_writer| {
                yaml_writer.write("type", "object");

                yaml_writer.write_upper_level("additionalProperties", |yaml_writer| {
                    write_array_element(yaml_writer, array_element);
                });
            });
        }
    }
}

fn write_simple_type(yaml_writer: &mut YamlWriter, param_type: &HttpSimpleType) {
    yaml_writer.write("type", param_type.as_swagger_type());
    yaml_writer.write("format", param_type.as_format());
}

fn write_array_element(yaml_writer: &mut YamlWriter, array_element: &ArrayElement) {
    yaml_writer.write("type", "array");

    yaml_writer.write_upper_level("items", |yaml_writer| {
        match array_element {
            ArrayElement::SimpleType(param_type) => write_simple_type(yaml_writer, param_type),
            ArrayElement::Object(object_type) => {
                super::object::write_reference_to_object(yaml_writer, object_type);
            }
            ArrayElement::Enum(enum_type) => {
                super::object::write_reference_to_object(yaml_writer, enum_type);
            }
        };
    });
}
