use async_trait::async_trait;
use hyper::Method;
use std::sync::Arc;

use my_http_server_core::{
    HttpContext, HttpFailResult, HttpOkResult, HttpServerMiddleware, HttpServerRequestFlow,
};

use super::{
    actions::{
        DeleteAction, GetAction, GetDescription, HandleHttpRequest, HttpAction, HttpActions,
        PostAction, PutAction,
    },
    documentation::{data_types::HttpObjectStructure, ShouldBeAuthorized},
    AuthErrorFactory, AuthorizationMap, HttpRoute,
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
            get: HttpActions::new(),
            post: HttpActions::new(),
            put: HttpActions::new(),
            delete: HttpActions::new(),
            http_objects: Vec::new(),
            authorization_map: AuthorizationMap::new(authorization),
            auth_error_factory,
        }
    }

    pub fn register_get_action<
        TGetAction: GetAction + HandleHttpRequest + GetDescription + Send + Sync + 'static,
    >(
        &mut self,
        action: Arc<TGetAction>,
    ) {
        let http_route = HttpRoute::new(action.get_route());

        if let Some(route_keys) = action.get_model_routes() {
            if let Err(err) = http_route.check_route_keys(&route_keys) {
                panic!("[GET]: {}", err)
            }
        }

        let result = self.get.register(HttpAction {
            handler: action.clone(),

            should_be_authorized: if let Some(desc) = action.get_description() {
                desc.input_params
                    .check_parameters(&Method::GET, http_route.route.as_str());
                desc.should_be_authorized
            } else {
                ShouldBeAuthorized::UseGlobal
            },
            http_route,
            description: action,
        });

        if let Err(err) = result {
            panic!("Failed to register GET action: {}", err);
        }
    }

    pub fn register_post_action<
        TPostAction: PostAction + HandleHttpRequest + GetDescription + Send + Sync + 'static,
    >(
        &mut self,
        action: Arc<TPostAction>,
    ) {
        let http_route = HttpRoute::new(action.get_route());
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
    }

    pub fn register_put_action<
        TPutAction: PutAction + HandleHttpRequest + GetDescription + Send + Sync + 'static,
    >(
        &mut self,
        action: Arc<TPutAction>,
    ) {
        let http_route = HttpRoute::new(action.get_route());
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
    }

    pub fn register_delete_action<
        TDeleteAction: DeleteAction + HandleHttpRequest + GetDescription + Send + Sync + 'static,
    >(
        &mut self,
        action: Arc<TDeleteAction>,
    ) {
        let http_route = HttpRoute::new(action.get_route());

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
        get_next: &mut HttpServerRequestFlow,
    ) -> Result<HttpOkResult, HttpFailResult> {
        match ctx.request.get_method() {
            &Method::GET => {
                {
                    if let Some(result) = self
                        .get
                        .handle_request(ctx, &self.authorization_map, &self.auth_error_factory)
                        .await
                    {
                        return result;
                    }
                }
                return get_next.next(ctx).await;
            }
            &Method::POST => {
                if let Some(result) = self
                    .post
                    .handle_request(ctx, &self.authorization_map, &self.auth_error_factory)
                    .await
                {
                    return result;
                } else {
                    return get_next.next(ctx).await;
                }
            }
            &Method::PUT => {
                if let Some(result) = self
                    .put
                    .handle_request(ctx, &self.authorization_map, &self.auth_error_factory)
                    .await
                {
                    return result;
                } else {
                    return get_next.next(ctx).await;
                }
            }
            &Method::DELETE => {
                if let Some(result) = self
                    .delete
                    .handle_request(ctx, &self.authorization_map, &self.auth_error_factory)
                    .await
                {
                    return result;
                } else {
                    return get_next.next(ctx).await;
                }
            }
            _ => {
                return get_next.next(ctx).await;
            }
        }
    }
}
