use std::collections::{BTreeMap, HashMap};

use crate::middlewares::{
    controllers::{
        documentation::{
            data_types::{ArrayElement, HttpDataType, HttpObjectStructure},
            HttpActionDescription,
        },
        ControllersMiddleware,
    },
    swagger::json_object_writer::JsonObjectWriter,
};

pub fn build(
    controllers: &ControllersMiddleware,
    path_descriptions: &BTreeMap<String, BTreeMap<String, HttpActionDescription>>,
) -> Option<JsonObjectWriter> {
    let mut result = JsonObjectWriter::as_object();
    let mut definitions = HashMap::new();

    for http_object in &controllers.http_objects {
        if !definitions.contains_key(http_object.struct_id.as_str()) {
            result.write_object(
                &http_object.struct_id,
                super::http_object_type::build(http_object),
            );
            definitions.insert(http_object.struct_id.to_string(), ());
        }
    }

    for (_, action_descriptions) in path_descriptions {
        for (_, action_description) in action_descriptions {
            populate_from_actions(&mut result, &mut definitions, action_description);
        }
    }

    if result.has_written() {
        Some(result)
    } else {
        None
    }
}

fn populate_from_actions(
    json_writer: &mut JsonObjectWriter,
    definitions: &mut HashMap<String, ()>,
    action_description: &HttpActionDescription,
) {
    for result in &action_description.results {
        populate_object_type(json_writer, definitions, &result.data_type);
    }

    if let Some(input_parameters) = &action_description.input_params {
        for in_param in input_parameters {
            populate_object_type(json_writer, definitions, &in_param.field.data_type);
        }
    }
}

fn populate_object_type(
    json_writer: &mut JsonObjectWriter,
    definitions: &mut HashMap<String, ()>,
    data_type: &HttpDataType,
) {
    match data_type {
        HttpDataType::SimpleType(_) => {}
        HttpDataType::Object(object_type) => {
            write_object_type(json_writer, definitions, object_type);
        }
        HttpDataType::ObjectId { struct_id: _ } => {}
        HttpDataType::ArrayOf(array_element) => {
            populate_array_type(json_writer, definitions, array_element);
        }
        HttpDataType::None => {}
    }
}

fn populate_array_type(
    json_writer: &mut JsonObjectWriter,
    definitions: &mut HashMap<String, ()>,
    array_element: &ArrayElement,
) {
    match array_element {
        ArrayElement::SimpleType(_) => {}
        ArrayElement::ObjectId { struct_id: _ } => {}
        ArrayElement::Object(object_type) => {
            write_object_type(json_writer, definitions, object_type)
        }
    }
}

fn write_object_type(
    json_writer: &mut JsonObjectWriter,
    definitions: &mut HashMap<String, ()>,
    object_type: &HttpObjectStructure,
) {
    if !definitions.contains_key(object_type.struct_id.as_str()) {
        json_writer.write_object(
            object_type.struct_id.as_ref(),
            super::http_object_type::build(object_type),
        );

        definitions.insert(object_type.struct_id.to_string(), ());
    }

    for field in &object_type.fields {
        populate_object_type(json_writer, definitions, &field.data_type);
    }
}
