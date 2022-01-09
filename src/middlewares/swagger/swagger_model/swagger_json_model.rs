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

    pub fn populate_operations(&mut self, controllers: &ControllersMiddleware) {
        for route_action in controllers.get.no_keys.values() {
            let mut path_model = SwaggerPathJsonModel::new();
            path_model.get = Some(SwaggerVerbDescription::new(
                route_action,
                route_action.action.get_in_parameters_description(),
            ));
            self.paths
                .insert(route_action.route.path.to_string(), path_model);
        }

        for route_action in &controllers.get.with_keys {
            let mut path_model = SwaggerPathJsonModel::new();
            path_model.get = Some(SwaggerVerbDescription::new(
                route_action,
                route_action.action.get_in_parameters_description(),
            ));
            self.paths
                .insert(route_action.route.path.to_string(), path_model);
        }

        for route_action in controllers.post.no_keys.values() {
            let mut path_model = SwaggerPathJsonModel::new();
            path_model.post = Some(SwaggerVerbDescription::new(
                route_action,
                route_action.action.get_in_parameters_description(),
            ));
            self.paths
                .insert(route_action.route.path.to_string(), path_model);
        }

        for route_action in &controllers.post.with_keys {
            let mut path_model = SwaggerPathJsonModel::new();
            path_model.post = Some(SwaggerVerbDescription::new(
                route_action,
                route_action.action.get_in_parameters_description(),
            ));
            self.paths
                .insert(route_action.route.path.to_string(), path_model);
        }

        for route_action in controllers.put.no_keys.values() {
            let mut path_model = SwaggerPathJsonModel::new();
            path_model.put = Some(SwaggerVerbDescription::new(
                route_action,
                route_action.action.get_in_parameters_description(),
            ));
            self.paths
                .insert(route_action.route.path.to_string(), path_model);
        }

        for route_action in &controllers.put.with_keys {
            let mut path_model = SwaggerPathJsonModel::new();
            path_model.put = Some(SwaggerVerbDescription::new(
                route_action,
                route_action.action.get_in_parameters_description(),
            ));
            self.paths
                .insert(route_action.route.path.to_string(), path_model);
        }

        for route_action in controllers.delete.no_keys.values() {
            let mut path_model = SwaggerPathJsonModel::new();
            path_model.delete = Some(SwaggerVerbDescription::new(
                route_action,
                route_action.action.get_in_parameters_description(),
            ));
            self.paths
                .insert(route_action.route.path.to_string(), path_model);
        }

        for route_action in &controllers.delete.with_keys {
            let mut path_model = SwaggerPathJsonModel::new();
            path_model.delete = Some(SwaggerVerbDescription::new(
                route_action,
                route_action.action.get_in_parameters_description(),
            ));
            self.paths
                .insert(route_action.route.path.to_string(), path_model);
        }
    }
}
