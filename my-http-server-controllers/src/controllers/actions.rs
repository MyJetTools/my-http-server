use std::sync::Arc;

use hyper::Method;
use my_http_server_core::{HttpContext, HttpFailResult, HttpOkResult};
use rust_extensions::date_time::DateTimeAsMicroseconds;

use super::{
    documentation::{HttpActionDescription, ShouldBeAuthorized},
    AuthErrorFactory, AuthorizationMap, HttpRoute,
};

pub trait GetAction {
    fn get_route(&self) -> &'static str;
    fn get_deprecated_routes(&self) -> Option<Vec<&'static str>>;
    fn get_model_routes(&self) -> Option<Vec<&'static str>>;
}

pub trait PostAction {
    fn get_route(&self) -> &'static str;
    fn get_deprecated_routes(&self) -> Option<Vec<&'static str>>;
    fn get_model_routes(&self) -> Option<Vec<&'static str>>;
}

pub trait PutAction {
    fn get_route(&self) -> &'static str;
    fn get_deprecated_routes(&self) -> Option<Vec<&'static str>>;
    fn get_model_routes(&self) -> Option<Vec<&'static str>>;
}

pub trait DeleteAction {
    fn get_route(&self) -> &'static str;
    fn get_deprecated_routes(&self) -> Option<Vec<&'static str>>;
    fn get_model_routes(&self) -> Option<Vec<&'static str>>;
}

pub trait OptionsAction {
    fn get_route(&self) -> &'static str;
    fn get_deprecated_routes(&self) -> Option<Vec<&'static str>>;
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
    pub deprecated: bool,
}

impl GetShouldBeAuthorized for HttpAction {
    fn get_should_be_authorized(&self) -> &ShouldBeAuthorized {
        &self.should_be_authorized
    }
}

pub struct HttpActions {
    actions: Vec<HttpAction>,
    pub action_verb: Method,
}

impl HttpActions {
    pub fn new(action_verb: Method) -> Self {
        Self {
            actions: Vec::new(),
            action_verb,
        }
    }

    pub fn register_action<
        TGetAction: HandleHttpRequest + GetDescription + Send + Sync + 'static,
    >(
        &mut self,
        action: Arc<TGetAction>,
        action_route: &str,
        model_routes: Option<Vec<&'static str>>,
        deprecated: bool,
    ) {
        let http_route = HttpRoute::new(action_route);

        if let Some(route_keys) = model_routes {
            if let Err(err) = http_route.check_route_keys(&route_keys) {
                panic!("[{}]: {}", self.action_verb, err)
            }
        }

        let result = self.register(HttpAction {
            handler: action.clone(),

            should_be_authorized: if let Some(desc) = action.get_description() {
                desc.input_params
                    .check_parameters(&self.action_verb, http_route.route.as_str());
                desc.should_be_authorized
            } else {
                ShouldBeAuthorized::UseGlobal
            },
            http_route,
            description: action,
            deprecated,
        });

        if let Err(err) = result {
            panic!("Failed to register [{}] action: {}", self.action_verb, err);
        }
    }

    fn register(&mut self, action: HttpAction) -> Result<(), String> {
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
        let now = DateTimeAsMicroseconds::now();
        for action in &self.actions {
            if action.http_route.is_my_path(&ctx.request.http_path) {
                ctx.process_name = Some(action.http_route.route.clone());
                match authorization_map.is_authorized(
                    action,
                    &ctx.credentials,
                    ctx.request.get_ip().get_real_ip(),
                    now,
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
