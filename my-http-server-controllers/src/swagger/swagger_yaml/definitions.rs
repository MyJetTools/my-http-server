use std::collections::{BTreeMap, HashMap};

use crate::controllers::{
    documentation::{
        data_types::{ArrayElement, HttpDataType, HttpObjectStructure},
        InputStructure,
    },
    ControllersMiddleware,
};

use super::{builder::SwaggerActionDescription, yaml_writer::YamlWriter};

pub fn build_and_write(
    yaml_writer: &mut YamlWriter,
    controllers: &ControllersMiddleware,
    path_descriptions: &BTreeMap<String, BTreeMap<String, SwaggerActionDescription>>,
) {
    yaml_writer.write_upper_level("schemas", |yaml_writer| {
        let mut definitions = HashMap::new();

        for http_object in &controllers.http_objects {
            write_object_type(yaml_writer, &mut definitions, http_object);
        }

        for (_, action_descriptions) in path_descriptions {
            for (_, action_description) in action_descriptions {
                for result in &action_description.description.results {
                    populate_object_type(yaml_writer, &mut definitions, &result.data_type);
                }

                if let Some(input_parameters) = action_description
                    .description
                    .input_params
                    .get_body_params()
                {
                    for in_param in input_parameters {
                        populate_object_type(
                            yaml_writer,
                            &mut definitions,
                            &in_param.field.data_type,
                        );
                    }
                }

                if let Some(input_parameters) = action_description
                    .description
                    .input_params
                    .get_non_body_params()
                {
                    for in_param in input_parameters {
                        populate_object_type(
                            yaml_writer,
                            &mut definitions,
                            &in_param.field.data_type,
                        );
                    }
                }

                if let Some(input_parameters) = action_description
                    .description
                    .input_params
                    .get_form_data_params()
                {
                    for in_param in input_parameters {
                        populate_object_type(
                            yaml_writer,
                            &mut definitions,
                            &in_param.field.data_type,
                        );
                    }
                }
            }
        }
    });
}

fn populate_object_type(
    yaml_writer: &mut YamlWriter,
    definitions: &mut HashMap<String, ()>,
    data_type: &HttpDataType,
) {
    match data_type {
        HttpDataType::SimpleType(_) => {}
        HttpDataType::Object(object_type) => {
            write_object_type(yaml_writer, definitions, object_type);
        }

        HttpDataType::ArrayOf(array_element) => {
            populate_array_type(yaml_writer, definitions, array_element);
        }
        HttpDataType::Enum(enum_structure) => {
            write_enum_type(yaml_writer, definitions, enum_structure);
        }

        HttpDataType::None => {}
        HttpDataType::DictionaryOf(array_element) => {
            populate_array_type(yaml_writer, definitions, array_element);
        }
        HttpDataType::DictionaryOfArray(array_element) => {
            populate_array_type(yaml_writer, definitions, array_element);
        }
    }
}

fn populate_array_type(
    yaml_writer: &mut YamlWriter,
    definitions: &mut HashMap<String, ()>,
    array_element: &ArrayElement,
) {
    match array_element {
        ArrayElement::SimpleType(_) => {}
        ArrayElement::Object(object_type) => {
            write_object_type(yaml_writer, definitions, object_type)
        }
        ArrayElement::Enum(enum_structure) => {
            write_enum_type(yaml_writer, definitions, enum_structure);
        }
    }
}

fn write_object_type(
    yaml_writer: &mut YamlWriter,
    definitions: &mut HashMap<String, ()>,
    object_type: &HttpObjectStructure,
) {
    let struct_id = object_type.get_struct_id();

    if !definitions.contains_key(struct_id.as_str()) {
        yaml_writer.write_upper_level(struct_id.as_str(), |yaml_writer| {
            super::http_object_type::build(yaml_writer, &object_type.main);
        });

        for field in &object_type.main.fields {
            populate_object_type(yaml_writer, definitions, &field.data_type);
        }

        definitions.insert(struct_id.to_string(), ());
    }

    if let Some(generic_data) = &object_type.generic {
        if !definitions.contains_key(generic_data.struct_id.as_str()) {
            yaml_writer.write_upper_level(generic_data.struct_id.as_str(), |yaml_writer| {
                super::http_object_type::build(yaml_writer, generic_data);
            });
            for field in &generic_data.fields {
                populate_object_type(yaml_writer, definitions, &field.data_type);
            }

            definitions.insert(generic_data.struct_id.as_str().to_string(), ());
        }
    }
}

fn write_enum_type(
    yaml_writer: &mut YamlWriter,
    definitions: &mut HashMap<String, ()>,
    enum_structure: &crate::controllers::documentation::data_types::HttpEnumStructure,
) {
    if definitions.contains_key(enum_structure.struct_id) {
        return;
    };

    yaml_writer.write_upper_level(enum_structure.struct_id.as_ref(), |yaml_writer| {
        super::http_enum_type::build(yaml_writer, enum_structure);
    });

    definitions.insert(enum_structure.struct_id.to_string(), ());
}
