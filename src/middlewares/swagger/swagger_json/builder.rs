use std::collections::BTreeMap;

use crate::middlewares::{
    controllers::{
        documentation::{HttpActionDescription, HttpActionDescriptionProvider},
        ControllersMiddleware,
    },
    swagger::json_object_writer::JsonObjectWriter,
};

pub fn build(
    controllers: &ControllersMiddleware,
    title: &str,
    version: &str,
    host: &str,
    scheme: &str,
) -> Vec<u8> {
    let mut json_object_writer = JsonObjectWriter::as_object();

    super::title::write(&mut json_object_writer, host, title, version);
    json_object_writer.write_object("scheme", super::schemes::build(scheme));

    let path_descriptions = build_paths_descriptions(controllers);

    if let Some(definitions) = super::definitions::build(controllers, &path_descriptions) {
        json_object_writer.write_object("definitions", definitions);
    }

    json_object_writer.write_object("paths", super::paths::build(&path_descriptions));

    json_object_writer.build()
}

fn build_paths_descriptions(
    controllers: &ControllersMiddleware,
) -> BTreeMap<String, BTreeMap<String, HttpActionDescription>> {
    let mut result = BTreeMap::new();

    for route_action in controllers.list_of_get_route_actions() {
        if let Some(description) = route_action.get_description() {
            if !result.contains_key(route_action.route.path.as_str()) {
                result.insert(route_action.route.path.to_string(), BTreeMap::new());
            }

            result
                .get_mut(route_action.route.path.as_str())
                .unwrap()
                .insert("get".to_string(), description);
        }
    }

    for route_action in controllers.list_of_post_route_actions() {
        if let Some(description) = route_action.get_description() {
            if !result.contains_key(route_action.route.path.as_str()) {
                result.insert(route_action.route.path.to_string(), BTreeMap::new());
            }

            result
                .get_mut(route_action.route.path.as_str())
                .unwrap()
                .insert("post".to_string(), description);
        }
    }

    for route_action in controllers.list_of_put_route_actions() {
        if let Some(description) = route_action.get_description() {
            if !result.contains_key(route_action.route.path.as_str()) {
                result.insert(route_action.route.path.to_string(), BTreeMap::new());
            }

            result
                .get_mut(route_action.route.path.as_str())
                .unwrap()
                .insert("put".to_string(), description);
        }
    }

    for route_action in controllers.list_of_delete_route_actions() {
        if let Some(description) = route_action.get_description() {
            if !result.contains_key(route_action.route.path.as_str()) {
                result.insert(route_action.route.path.to_string(), BTreeMap::new());
            }

            result
                .get_mut(route_action.route.path.as_str())
                .unwrap()
                .insert("delete".to_string(), description);
        }
    }

    result
}
