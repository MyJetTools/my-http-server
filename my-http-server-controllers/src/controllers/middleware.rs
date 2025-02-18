use async_trait::async_trait;
use hyper::Method;
use std::sync::Arc;

use my_http_server_core::{HttpContext, HttpFailResult, HttpOkResult, HttpServerMiddleware};

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

        /*
        if let Some(route_keys) = action.get_model_routes() {
            if let Err(err) = http_route.check_route_keys(&route_keys) {
                panic!("[POST]: {}", err)
            }
        }

        let result = self.post.register(HttpAction {
            handler: action.clone(),

            should_be_authorized: if let Some(desc) = action.get_description() {
                desc.input_params
                    .check_parameters(&Method::POST, http_route.route.as_str());
                desc.should_be_authorized
            } else {
                ShouldBeAuthorized::UseGlobal
            },
            http_route,
            description: action,
        });

        if let Err(err) = result {
            panic!("Failed to register POST action: {}", err);
        }
         */
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

        /*
        if let Some(route_keys) = action.get_model_routes() {
            if let Err(err) = http_route.check_route_keys(&route_keys) {
                panic!("[PUT]: {}", err)
            }
        }

        let result = self.put.register(HttpAction {
            handler: action.clone(),

            should_be_authorized: if let Some(desc) = action.get_description() {
                desc.input_params
                    .check_parameters(&Method::PUT, http_route.route.as_str());
                desc.should_be_authorized
            } else {
                ShouldBeAuthorized::UseGlobal
            },
            http_route,
            description: action,
        });

        if let Err(err) = result {
            panic!("Failed to register PUT action: {}", err);
        }
         */
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

        /*
        if let Some(route_keys) = action.get_model_routes() {
            if let Err(err) = http_route.check_route_keys(&route_keys) {
                panic!("[DELETE]: {}", err)
            }
        }

        let result = self.delete.register(HttpAction {
            handler: action.clone(),

            should_be_authorized: if let Some(desc) = action.get_description() {
                desc.input_params
                    .check_parameters(&Method::DELETE, http_route.route.as_str());
                desc.should_be_authorized
            } else {
                ShouldBeAuthorized::UseGlobal
            },
            http_route,
            description: action,
        });

        if let Err(err) = result {
            panic!("Failed to register DELETE action: {}", err);
        }
         */
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
            _ => None,
        }
    }
}
