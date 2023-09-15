use crate::controllers::documentation::{
    ArrayElement, HttpDataType, HttpEnumStructure, HttpField, HttpObjectStructure, HttpSimpleType,
};

use super::yaml_writer::YamlWriter;

pub fn write(yaml_writer: &mut YamlWriter, field: &HttpField) {
    match &field.data_type {
        HttpDataType::SimpleType(simple_type) => {
            yaml_writer.write_upper_level(field.name.as_str(), |yaml_writer| {
                write_body_simple_type(yaml_writer, simple_type);
            });
        }
        HttpDataType::Object(object) => {
            yaml_writer.write_upper_level(field.name.as_str(), |yaml_writer| {
                write_body_object_type(yaml_writer, object);
            });
        }
        HttpDataType::ArrayOf(array_el) => {
            yaml_writer.write_upper_level(field.name.as_str(), |yaml_writer| {
                write_body_array_type(yaml_writer, array_el);
            });
        }
        HttpDataType::DictionaryOf(array_el) => match array_el {
            crate::controllers::documentation::ArrayElement::SimpleType(simple_type) => {
                yaml_writer.write_upper_level(field.name.as_str(), |yaml_writer| {
                    yaml_writer.write("type", "object");
                    yaml_writer.write_upper_level("additionalProperties", |yaml_writer| {
                        write_body_simple_type(yaml_writer, simple_type);
                    });
                });
            }
            crate::controllers::documentation::ArrayElement::Object(obj) => {
                yaml_writer.write_upper_level(field.name.as_str(), |yaml_writer| {
                    yaml_writer.write("type", "object");
                    yaml_writer.write_upper_level("additionalProperties", |yaml_writer| {
                        yaml_writer.write_upper_level("items", |yaml_writer| {
                            write_body_object_type(yaml_writer, obj);
                        });
                    });
                });
            }

            crate::controllers::documentation::ArrayElement::Enum(enum_type) => {
                panic!(
                    "Enum in dictionary of enum is not supported. {:?}",
                    enum_type
                );
            }
        },
        HttpDataType::DictionaryOfArray(array_el) => {
            yaml_writer.write_upper_level(field.name.as_str(), |yaml_writer| {
                yaml_writer.write("type", "object");
                yaml_writer.write_upper_level("additionalProperties", |yaml_writer| {
                    write_body_array_type(yaml_writer, array_el);
                });
            });
        }
        HttpDataType::Enum(enum_data) => {
            yaml_writer.write_upper_level(field.name.as_str(), |yaml_writer| {
                write_enum(yaml_writer, enum_data);
            });
        }
        HttpDataType::None => {}
    }
}

fn write_body_simple_type(yaml_writer: &mut YamlWriter, simple_type: &HttpSimpleType) {
    yaml_writer.write("type", simple_type.as_swagger_type());
}

fn write_enum(yaml_writer: &mut YamlWriter, enum_data: &HttpEnumStructure) {
    match &enum_data.enum_type {
        crate::controllers::documentation::EnumType::Integer => {
            yaml_writer.write("type", "integer");
        }
        crate::controllers::documentation::EnumType::String => {
            yaml_writer.write("type", "string");
        }
    }
}

fn write_body_object_type(yaml_writer: &mut YamlWriter, object: &HttpObjectStructure) {
    yaml_writer.write("type", "object");
    yaml_writer.write_upper_level("properties", |yaml_writer| {
        for obj_field in &object.main.fields {
            write(yaml_writer, obj_field);
        }
    });
}

fn write_body_array_type(yaml_writer: &mut YamlWriter, array_el: &ArrayElement) {
    match array_el {
        crate::controllers::documentation::ArrayElement::SimpleType(simple_type) => {
            yaml_writer.write("type", "array");
            yaml_writer.write_upper_level("items", |yaml_writer| {
                write_body_simple_type(yaml_writer, simple_type);
            });
        }
        crate::controllers::documentation::ArrayElement::Object(obj) => {
            yaml_writer.write("type", "array");
            yaml_writer.write_upper_level("items", |yaml_writer| {
                write_body_object_type(yaml_writer, obj);
            });
        }

        crate::controllers::documentation::ArrayElement::Enum(enum_type) => {
            panic!("Enum in array not supported as body type. {:?}", enum_type);
        }
    }
}
