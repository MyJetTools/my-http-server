use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::middlewares::controllers::ControllersMiddleware;

use super::{
    path::{SwaggerPathJsonModel, SwaggerVerbDescription},
    SwaggerInfoJsonModel,
};

#[derive(Serialize, Deserialize, Debug)]
pub struct SwaggerJsonModel {
    #[serde(rename = "x-generator")]
    generator: String,
    swagger: String,
    info: SwaggerInfoJsonModel,
    host: String,
    scheme: Vec<String>,
    paths: BTreeMap<String, SwaggerPathJsonModel>,
}

impl SwaggerJsonModel {
    pub fn new(title: String, version: String, host: String, scheme: String) -> Self {
        Self {
            generator: "My-Http-Server-Generator".to_string(),
            swagger: "2.0".to_string(),
            info: SwaggerInfoJsonModel { title, version },
            host,
            scheme: vec![scheme],
            paths: BTreeMap::new(),
        }
    }

    fn get_or_create(&mut self, route_path: &str) -> &mut SwaggerPathJsonModel {
        if !self.paths.contains_key(route_path) {
            self.paths
                .insert(route_path.to_string(), SwaggerPathJsonModel::new());
        }

        return self.paths.get_mut(route_path).unwrap();
    }

    pub fn populate_operations(&mut self, controllers: &ControllersMiddleware) {
        for route_action in controllers.get.no_keys.values() {
            if let Some(action_description) = route_action.action.get_controller_description() {
                let path_model = self.get_or_create(route_action.route.path.as_str());
                path_model.get = Some(SwaggerVerbDescription::new(
                    action_description,
                    route_action.action.get_in_parameters_description(),
                ));
            }
        }

        for route_action in &controllers.get.with_keys {
            if let Some(action_description) = route_action.action.get_controller_description() {
                let path_model = self.get_or_create(route_action.route.path.as_str());
                path_model.get = Some(SwaggerVerbDescription::new(
                    action_description,
                    route_action.action.get_in_parameters_description(),
                ));
            }
        }

        for route_action in controllers.post.no_keys.values() {
            if let Some(action_description) = route_action.action.get_controller_description() {
                let path_model = self.get_or_create(route_action.route.path.as_str());
                path_model.post = Some(SwaggerVerbDescription::new(
                    action_description,
                    route_action.action.get_in_parameters_description(),
                ));
            }
        }

        for route_action in &controllers.post.with_keys {
            if let Some(action_description) = route_action.action.get_controller_description() {
                let path_model = self.get_or_create(route_action.route.path.as_str());

                path_model.post = Some(SwaggerVerbDescription::new(
                    action_description,
                    route_action.action.get_in_parameters_description(),
                ));
            }
        }

        for route_action in controllers.put.no_keys.values() {
            if let Some(action_description) = route_action.action.get_controller_description() {
                let path_model = self.get_or_create(route_action.route.path.as_str());
                path_model.put = Some(SwaggerVerbDescription::new(
                    action_description,
                    route_action.action.get_in_parameters_description(),
                ));
            }
        }

        for route_action in &controllers.put.with_keys {
            if let Some(action_description) = route_action.action.get_controller_description() {
                let path_model = self.get_or_create(route_action.route.path.as_str());
                path_model.put = Some(SwaggerVerbDescription::new(
                    action_description,
                    route_action.action.get_in_parameters_description(),
                ));
            }
        }

        for route_action in controllers.delete.no_keys.values() {
            if let Some(action_description) = route_action.action.get_controller_description() {
                let path_model = self.get_or_create(route_action.route.path.as_str());
                path_model.delete = Some(SwaggerVerbDescription::new(
                    action_description,
                    route_action.action.get_in_parameters_description(),
                ));
            }
        }

        for route_action in &controllers.delete.with_keys {
            if let Some(action_description) = route_action.action.get_controller_description() {
                let path_model = self.get_or_create(route_action.route.path.as_str());
                path_model.delete = Some(SwaggerVerbDescription::new(
                    action_description,
                    route_action.action.get_in_parameters_description(),
                ));
            }
        }
    }
}
