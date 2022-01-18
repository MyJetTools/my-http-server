use std::collections::BTreeMap;

use crate::middlewares::{
    controllers::documentation::HttpActionDescription,
    swagger::json_object_writer::JsonObjectWriter,
};

pub fn build(
    actions: &BTreeMap<String, BTreeMap<String, HttpActionDescription>>,
) -> JsonObjectWriter {
    let mut result = JsonObjectWriter::as_object();
    for (path, actions) in actions {
        let mut path_object = JsonObjectWriter::as_object();
        for (verb, action_description) in actions {
            path_object.write_object(verb, super::verb_description::build(action_description));
        }
        result.write_object(path.as_str(), path_object);
    }

    result
}
