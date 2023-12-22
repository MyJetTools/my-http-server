use crate::{
    controllers::documentation::{in_parameters::HttpInputParameter, ArrayElement, HttpDataType},
    swagger::swagger_yaml::yaml_writer::YamlWriter,
};

pub fn write_query_input_param(yaml_writer: &mut YamlWriter, input_param: &HttpInputParameter) {
    yaml_writer.write("name", input_param.field.get_query_field_name().as_str());
    yaml_writer.write_as_str("description", input_param.description.as_str());

    yaml_writer.write_upper_level("schema", |yaml_writer| {
        match &input_param.field.data_type {
            HttpDataType::SimpleType(simple_type) => {
                yaml_writer.write("type", simple_type.as_swagger_type());
                yaml_writer.write("format", simple_type.as_format());
            }
            HttpDataType::Object(_) => {
                panic!("Object type is not supported for non body parameter")
            }
            HttpDataType::ArrayOf(array_el) => write_array_item_of(yaml_writer, array_el),
            HttpDataType::DictionaryOf(_) => {
                panic!("Dictionary can not be used as a non body parameter")
            }
            HttpDataType::DictionaryOfArray(_) => {
                panic!("Dictionary of array can not be used as a non body parameter")
            }
            HttpDataType::Enum(enum_data) => {
                super::super::object::write_reference_to_object(yaml_writer, enum_data);
            }
            HttpDataType::None => {
                panic!("Somehow we have non parameter")
            }
        };
    });

    yaml_writer.write_bool("required", input_param.field.required);
}

fn write_array_item_of(yaml_writer: &mut YamlWriter, array_el: &ArrayElement) {
    match array_el {
        crate::controllers::documentation::ArrayElement::SimpleType(simple_type) => {
            super::write_array_input_param(yaml_writer, simple_type);
        }
        crate::controllers::documentation::ArrayElement::Object(_) => {
            panic!("Array of object type is not supported for non body parameter")
        }

        crate::controllers::documentation::ArrayElement::Enum(enum_data) => {
            super::write_array_enum_case(yaml_writer, enum_data);
        }
    }
}
