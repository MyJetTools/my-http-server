use async_trait::async_trait;
use hyper::Method;
use std::sync::Arc;

use my_http_server_core::{HttpContext, HttpFailResult, HttpOkResult, HttpServerMiddleware};

use crate::actions::OptionsAction;

use super::{
    actions::{
        DeleteAction, GetAction, GetDescription, HandleHttpRequest, HttpAction, HttpActions,
        PostAction, PutAction,
    },
    documentation::data_types::HttpObjectStructure,
    AuthErrorFactory, AuthorizationMap,
};

use super::ControllersAuthorization;

pub struct ControllersMiddleware {
    pub get: HttpActions,
    pub post: HttpActions,
    pub put: HttpActions,
    pub delete: HttpActions,
    pub options: HttpActions,
    pub http_objects: Vec<HttpObjectStructure>,
    pub authorization_map: AuthorizationMap,
    pub auth_error_factory: Option<Arc<dyn AuthErrorFactory + Send + Sync + 'static>>,
}

impl ControllersMiddleware {
    pub fn new(
        authorization: Option<ControllersAuthorization>,
        auth_error_factory: Option<Arc<dyn AuthErrorFactory + Send + Sync + 'static>>,
    ) -> Self {
        Self {
            get: HttpActions::new(Method::GET),
            post: HttpActions::new(Method::POST),
            put: HttpActions::new(Method::PUT),
            delete: HttpActions::new(Method::DELETE),
            options: HttpActions::new(Method::OPTIONS),
            http_objects: Vec::new(),
            authorization_map: AuthorizationMap::new(authorization),
            auth_error_factory,
        }
    }

    pub fn update_authorization_map(&mut self, authorization: ControllersAuthorization) {
        self.authorization_map.global_authorization = Some(authorization);
    }

    pub fn update_auth_error_factory(
        &mut self,
        value: Arc<dyn AuthErrorFactory + Send + Sync + 'static>,
    ) {
        self.auth_error_factory = Some(value);
    }

    pub fn register_get_action<
        TGetAction: GetAction + HandleHttpRequest + GetDescription + Clone + Send + Sync + 'static,
    >(
        &mut self,
        action: Arc<TGetAction>,
    ) {
        let route = action.get_route();

        let model_routes = action.get_model_routes();

        self.get
            .register_action(action.clone(), route, model_routes.clone(), false);

        if let Some(deprecated_rotes) = action.get_deprecated_routes() {
            for deprecated_route in deprecated_rotes {
                self.get.register_action(
                    action.clone(),
                    deprecated_route,
                    model_routes.clone(),
                    true,
                );
            }
        }
    }

    pub fn register_post_action<
        TPostAction: PostAction + HandleHttpRequest + GetDescription + Clone + Send + Sync + 'static,
    >(
        &mut self,
        action: Arc<TPostAction>,
    ) {
        let route = action.get_route();

        let model_routes = action.get_model_routes();

        self.post
            .register_action(action.clone(), route, model_routes.clone(), false);

        if let Some(deprecated_rotes) = action.get_deprecated_routes() {
            for deprecated_route in deprecated_rotes {
                self.post.register_action(
                    action.clone(),
                    deprecated_route,
                    model_routes.clone(),
                    true,
                );
            }
        }
    }

    pub fn register_put_action<
        TPutAction: PutAction + HandleHttpRequest + GetDescription + Send + Sync + 'static,
    >(
        &mut self,
        action: Arc<TPutAction>,
    ) {
        let route = action.get_route();

        let model_routes = action.get_model_routes();

        self.put
            .register_action(action.clone(), route, model_routes.clone(), false);

        if let Some(deprecated_rotes) = action.get_deprecated_routes() {
            for deprecated_route in deprecated_rotes {
                self.put.register_action(
                    action.clone(),
                    deprecated_route,
                    model_routes.clone(),
                    true,
                );
            }
        }
    }

    pub fn register_delete_action<
        TDeleteAction: DeleteAction + HandleHttpRequest + GetDescription + Send + Sync + 'static,
    >(
        &mut self,
        action: Arc<TDeleteAction>,
    ) {
        let route = action.get_route();

        let model_routes = action.get_model_routes();

        self.delete
            .register_action(action.clone(), route, model_routes.clone(), false);

        if let Some(deprecated_rotes) = action.get_deprecated_routes() {
            for deprecated_route in deprecated_rotes {
                self.delete.register_action(
                    action.clone(),
                    deprecated_route,
                    model_routes.clone(),
                    true,
                );
            }
        }
    }

    pub fn register_options_action<
        TOptionsAction: OptionsAction + HandleHttpRequest + GetDescription + Send + Sync + 'static,
    >(
        &mut self,
        action: Arc<TOptionsAction>,
    ) {
        let route = action.get_route();

        let model_routes = action.get_model_routes();

        self.options
            .register_action(action.clone(), route, model_routes.clone(), false);

        if let Some(deprecated_rotes) = action.get_deprecated_routes() {
            for deprecated_route in deprecated_rotes {
                self.options.register_action(
                    action.clone(),
                    deprecated_route,
                    model_routes.clone(),
                    true,
                );
            }
        }
    }

    pub fn list_of_get_route_actions(&self) -> &Vec<HttpAction> {
        self.get.get_actions()
    }

    pub fn list_of_post_route_actions(&self) -> &Vec<HttpAction> {
        self.post.get_actions()
    }

    pub fn list_of_put_route_actions(&self) -> &Vec<HttpAction> {
        self.put.get_actions()
    }

    pub fn list_of_delete_route_actions<'s>(&self) -> &Vec<HttpAction> {
        self.delete.get_actions()
    }

    pub fn list_of_options_route_actions<'s>(&self) -> &Vec<HttpAction> {
        self.options.get_actions()
    }
}

#[async_trait]
impl HttpServerMiddleware for ControllersMiddleware {
    async fn handle_request(
        &self,
        ctx: &mut HttpContext,
    ) -> Option<Result<HttpOkResult, HttpFailResult>> {
        match ctx.request.method {
            Method::GET => {
                self.get
                    .handle_request(ctx, &self.authorization_map, &self.auth_error_factory)
                    .await
            }
            Method::POST => {
                self.post
                    .handle_request(ctx, &self.authorization_map, &self.auth_error_factory)
                    .await
            }
            Method::PUT => {
                self.put
                    .handle_request(ctx, &self.authorization_map, &self.auth_error_factory)
                    .await
            }
            Method::DELETE => {
                self.delete
                    .handle_request(ctx, &self.authorization_map, &self.auth_error_factory)
                    .await
            }
            Method::OPTIONS => {
                self.options
                    .handle_request(ctx, &self.authorization_map, &self.auth_error_factory)
                    .await
            }
            _ => None,
        }
    }
}
