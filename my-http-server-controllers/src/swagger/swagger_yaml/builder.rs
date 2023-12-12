use std::collections::BTreeMap;

use crate::controllers::{
    documentation::{out_results::HttpResult, HttpActionDescription},
    ControllersMiddleware,
};

use super::yaml_writer::YamlWriter;

pub fn build(
    controllers: &ControllersMiddleware,
    title: &str,
    version: &str,
    host: &str,
    scheme: &str,
    global_fail_results: Option<Vec<HttpResult>>,
) -> Vec<u8> {
    let mut yaml_writer = YamlWriter::new();

    yaml_writer.write("openapi", "3.0.0");

    yaml_writer.write_upper_level("info", |yaml_writer| {
        yaml_writer.write("title", title);
        yaml_writer.write("version", version);
    });

    yaml_writer.write_upper_level("servers", |yaml_writer| {
        yaml_writer.write("- url", format!("{}://{}", scheme, host).as_str());
    });

    let path_descriptions = build_paths_descriptions(controllers, global_fail_results);

    yaml_writer.write_upper_level("components", |yaml_writer| {
        super::definitions::build_and_write(yaml_writer, controllers, &path_descriptions);

        super::security_definitions::build(
            yaml_writer,
            &controllers.authorization_map.global_authorization,
        );
    });

    super::paths::build(&mut yaml_writer, &path_descriptions, controllers);

    yaml_writer.build()
}

pub struct SwaggerActionDescription<'s> {
    pub deprecated: bool,
    pub description: HttpActionDescription<'s>,
}

fn build_paths_descriptions(
    controllers: &ControllersMiddleware,
    global_fail_results: Option<Vec<HttpResult>>,
) -> BTreeMap<String, BTreeMap<String, SwaggerActionDescription>> {
    let mut result = BTreeMap::new();

    for http_action in controllers.list_of_get_route_actions() {
        if let Some(description) = http_action.description.get_description() {
            if !result.contains_key(http_action.http_route.route.as_str()) {
                result.insert(http_action.http_route.route.to_string(), BTreeMap::new());
            }

            result
                .get_mut(http_action.http_route.route.as_str())
                .unwrap()
                .insert(
                    "get".to_string(),
                    SwaggerActionDescription {
                        deprecated: http_action.deprecated,
                        description: description,
                    },
                );
        }
    }

    for http_action in controllers.list_of_post_route_actions() {
        if let Some(description) = http_action.description.get_description() {
            if !result.contains_key(http_action.http_route.route.as_str()) {
                result.insert(http_action.http_route.route.to_string(), BTreeMap::new());
            }

            result
                .get_mut(http_action.http_route.route.as_str())
                .unwrap()
                .insert(
                    "post".to_string(),
                    SwaggerActionDescription {
                        deprecated: http_action.deprecated,
                        description: description,
                    },
                );
        }
    }

    for route_action in controllers.list_of_put_route_actions() {
        if let Some(description) = route_action.description.get_description() {
            if !result.contains_key(route_action.http_route.route.as_str()) {
                result.insert(route_action.http_route.route.to_string(), BTreeMap::new());
            }

            result
                .get_mut(route_action.http_route.route.as_str())
                .unwrap()
                .insert(
                    "put".to_string(),
                    SwaggerActionDescription {
                        deprecated: route_action.deprecated,
                        description: description,
                    },
                );
        }
    }

    for route_action in controllers.list_of_delete_route_actions() {
        if let Some(description) = route_action.description.get_description() {
            if !result.contains_key(route_action.http_route.route.as_str()) {
                result.insert(route_action.http_route.route.to_string(), BTreeMap::new());
            }

            result
                .get_mut(route_action.http_route.route.as_str())
                .unwrap()
                .insert(
                    "delete".to_string(),
                    SwaggerActionDescription {
                        deprecated: route_action.deprecated,
                        description: description,
                    },
                );
        }
    }

    if let Some(global_path_description) = global_fail_results {
        for verbs in result.values_mut() {
            for action in verbs.values_mut() {
                for global_fail_result in &global_path_description {
                    action.description.results.push(global_fail_result.clone());
                }
            }
        }
    }

    result
}
