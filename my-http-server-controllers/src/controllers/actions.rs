use std::sync::Arc;

use my_http_server_core::{HttpContext, HttpFailResult, HttpOkResult};

use super::{
    documentation::{HttpActionDescription, ShouldBeAuthorized},
    AuthErrorFactory, AuthorizationMap, HttpRoute,
};

pub trait GetAction {
    fn get_route(&self) -> &str;
    fn get_model_routes(&self) -> Option<Vec<&'static str>>;
}

pub trait PostAction {
    fn get_route(&self) -> &str;
    fn get_model_routes(&self) -> Option<Vec<&'static str>>;
}

pub trait PutAction {
    fn get_route(&self) -> &str;
    fn get_model_routes(&self) -> Option<Vec<&'static str>>;
}

pub trait DeleteAction {
    fn get_route(&self) -> &str;
    fn get_model_routes(&self) -> Option<Vec<&'static str>>;
}

#[async_trait::async_trait]
pub trait HandleHttpRequest {
    async fn handle_request(
        &self,
        http_route: &HttpRoute,
        ctx: &mut HttpContext,
    ) -> Result<HttpOkResult, HttpFailResult>;
}

pub trait GetDescription {
    fn get_description(&self) -> Option<HttpActionDescription>;
}

pub trait GetShouldBeAuthorized {
    fn get_should_be_authorized(&self) -> &ShouldBeAuthorized;
}

pub struct HttpAction {
    pub handler: Arc<dyn HandleHttpRequest + Send + Sync + 'static>,
    pub http_route: HttpRoute,
    pub description: Arc<dyn GetDescription + Send + Sync + 'static>,
    pub should_be_authorized: ShouldBeAuthorized,
}

impl GetShouldBeAuthorized for HttpAction {
    fn get_should_be_authorized(&self) -> &ShouldBeAuthorized {
        &self.should_be_authorized
    }
}

pub struct HttpActions {
    actions: Vec<HttpAction>,
}

impl HttpActions {
    pub fn new() -> Self {
        Self {
            actions: Vec::new(),
        }
    }

    pub fn register(&mut self, action: HttpAction) -> Result<(), String> {
        for registered_action in &self.actions {
            if registered_action.http_route.route.to_lowercase()
                == action.http_route.route.to_lowercase()
            {
                return Err(format!(
                    "Route {} is already registered",
                    action.http_route.route
                ));
            }
        }

        self.actions.push(action);

        Ok(())
    }

    pub async fn handle_request(
        &self,
        ctx: &mut HttpContext,
        authorization_map: &AuthorizationMap,
        auth_error_factory: &Option<Arc<dyn AuthErrorFactory + Send + Sync + 'static>>,
    ) -> Option<Result<HttpOkResult, HttpFailResult>> {
        for action in &self.actions {
            if action.http_route.is_my_path(&ctx.request.http_path) {
                match authorization_map.is_authorized(
                    action,
                    &ctx.credentials,
                    ctx.request.get_ip().get_real_ip(),
                ) {
                    super::AuthorizationResult::Allowed => {
                        return Some(action.handler.handle_request(&action.http_route, ctx).await);
                    }
                    super::AuthorizationResult::NotAuthenticated => {
                        if let Some(result) = auth_error_factory {
                            return Some(Err(result.get_not_authenticated()));
                        } else {
                            return Some(Err(HttpFailResult::as_unauthorized(Some(
                                "No session credentials are found".to_string(),
                            ))));
                        }
                    }
                    super::AuthorizationResult::NotAuthorized(claim_name) => {
                        if let Some(result) = auth_error_factory {
                            return Some(Err(result.get_not_authorized(claim_name)));
                        } else {
                            return Some(Err(HttpFailResult::as_unauthorized(None)));
                        }
                    }
                }
            }
        }

        None
    }

    pub fn get_actions(&self) -> &Vec<HttpAction> {
        &self.actions
    }
}
