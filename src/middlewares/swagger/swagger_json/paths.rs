use std::collections::BTreeMap;

use crate::middlewares::{
    controllers::ControllersMiddleware, swagger::json_object_writer::JsonObjectWriter,
};

pub fn build(controllers: &ControllersMiddleware) -> JsonObjectWriter {
    let mut paths = get_actions(controllers);

    for route_action in controllers.list_of_get_route_actions() {
        if let Some(action_description) = route_action.action.get_description() {
            let path_object = paths.get_mut(route_action.route.path.as_str()).unwrap();
            path_object.write_object("get", super::verb_description::build(&action_description));
        }
    }

    for route_action in controllers.list_of_post_route_actions() {
        if let Some(action_description) = route_action.action.get_description() {
            let path_object = paths.get_mut(route_action.route.path.as_str()).unwrap();
            path_object.write_object("post", super::verb_description::build(&action_description));
        }
    }

    for route_action in controllers.list_of_put_route_actions() {
        if let Some(action_description) = route_action.action.get_description() {
            let path_object = paths.get_mut(route_action.route.path.as_str()).unwrap();
            path_object.write_object("put", super::verb_description::build(&action_description));
        }
    }

    for route_action in controllers.list_of_delete_route_actions() {
        if let Some(action_description) = route_action.action.get_description() {
            let path_object = paths.get_mut(route_action.route.path.as_str()).unwrap();
            path_object.write_object(
                "delete",
                super::verb_description::build(&action_description),
            );
        }
    }

    let mut result = JsonObjectWriter::as_object();
    for (path, obj) in paths {
        result.write_object(path.as_str(), obj);
    }
    result
}

fn get_actions(controllers: &ControllersMiddleware) -> BTreeMap<String, JsonObjectWriter> {
    let mut result = BTreeMap::new();

    for route_action in controllers.list_of_get_route_actions() {
        if !result.contains_key(route_action.route.path.as_str()) {
            result.insert(
                route_action.route.path.to_string(),
                JsonObjectWriter::as_object(),
            );
        }
    }

    for route_action in controllers.list_of_post_route_actions() {
        if !result.contains_key(route_action.route.path.as_str()) {
            result.insert(
                route_action.route.path.to_string(),
                JsonObjectWriter::as_object(),
            );
        }
    }

    for route_action in controllers.list_of_put_route_actions() {
        if !result.contains_key(route_action.route.path.as_str()) {
            result.insert(
                route_action.route.path.to_string(),
                JsonObjectWriter::as_object(),
            );
        }
    }

    for route_action in controllers.list_of_delete_route_actions() {
        if !result.contains_key(route_action.route.path.as_str()) {
            result.insert(
                route_action.route.path.to_string(),
                JsonObjectWriter::as_object(),
            );
        }
    }

    result
}
