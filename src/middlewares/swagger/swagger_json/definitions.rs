use std::collections::BTreeMap;

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
    path_descriptions: &BTreeMap<String, Vec<(String, HttpActionDescription)>>,
) -> Option<JsonObjectWriter> {
    let mut result = JsonObjectWriter::as_object();

    for http_object in &controllers.http_objects {
        result.write_object(
            &http_object.struct_id,
            super::http_object_type::build(http_object),
        );
    }

    for (_, action_descriptions) in path_descriptions {
        for (_, action_description) in action_descriptions {
            populate_from_actions(&mut result, action_description);
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
    action_description: &HttpActionDescription,
) {
    for result in &action_description.results {
        populate_object_type(json_writer, &result.data_type);
    }

    if let Some(input_parameters) = &action_description.input_params {
        for in_param in input_parameters {
            populate_object_type(json_writer, &in_param.field.data_type);
        }
    }
}

fn populate_object_type(json_writer: &mut JsonObjectWriter, data_type: &HttpDataType) {
    match data_type {
        HttpDataType::SimpleType(_) => {}
        HttpDataType::Object(object_type) => {
            write_object_type(json_writer, object_type);
        }
        HttpDataType::ObjectId { struct_id: _ } => {}
        HttpDataType::ArrayOf(array_element) => {
            populate_array_type(json_writer, array_element);
        }
        HttpDataType::None => {}
    }
}

fn populate_array_type(json_writer: &mut JsonObjectWriter, array_element: &ArrayElement) {
    match array_element {
        ArrayElement::SimpleType(_) => {}
        ArrayElement::ObjectId { struct_id: _ } => {}
        ArrayElement::Object(object_type) => write_object_type(json_writer, object_type),
    }
}

fn write_object_type(json_writer: &mut JsonObjectWriter, object_type: &HttpObjectStructure) {
    json_writer.write_object(
        object_type.struct_id.as_ref(),
        super::http_object_type::build(object_type),
    );

    for field in &object_type.fields {
        populate_object_type(json_writer, &field.data_type);
    }
}
