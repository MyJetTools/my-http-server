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
    definitions: Option<BTreeMap<String, SwaggerPathJsonModel>>,
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
            definitions: None,
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
        for route_action in controllers.list_of_get_route_actions() {
            if let Some(action_description) = route_action.action.get_description() {
                let path_model = self.get_or_create(route_action.route.path.as_str());
                path_model.get = Some(SwaggerVerbDescription::new(action_description));
            }
        }

        for route_action in controllers.list_of_post_route_actions() {
            if let Some(action_description) = route_action.action.get_description() {
                let path_model = self.get_or_create(route_action.route.path.as_str());
                path_model.get = Some(SwaggerVerbDescription::new(action_description));
            }
        }

        for route_action in controllers.list_of_put_route_actions() {
            if let Some(action_description) = route_action.action.get_description() {
                let path_model = self.get_or_create(route_action.route.path.as_str());
                path_model.get = Some(SwaggerVerbDescription::new(action_description));
            }
        }

        for route_action in controllers.list_of_delete_route_actions() {
            if let Some(action_description) = route_action.action.get_description() {
                let path_model = self.get_or_create(route_action.route.path.as_str());
                path_model.get = Some(SwaggerVerbDescription::new(action_description));
            }
        }
    }
}
