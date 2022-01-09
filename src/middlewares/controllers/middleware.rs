use async_trait::async_trait;
use std::sync::Arc;

use crate::{HttpContext, HttpFailResult, HttpServerMiddleware, MiddleWareResult};
use hyper::Method;

use super::{
    actions::{DeleteAction, GetAction, PostAction, PutAction},
    http_vebs::delete::*,
    http_vebs::get::*,
    http_vebs::post::*,
    http_vebs::put::*,
};

pub struct ControllersMiddleware {
    pub get: GetRoute,
    pub post: PostRoute,
    pub put: PutRoute,
    pub delete: DeleteRoute,
}

impl ControllersMiddleware {
    pub fn new() -> Self {
        Self {
            get: GetRoute::new(),
            post: PostRoute::new(),
            put: PutRoute::new(),
            delete: DeleteRoute::new(),
        }
    }

    pub fn register_get_action(
        &mut self,

        route: &str,
        action: Arc<dyn GetAction + Send + Sync + 'static>,
    ) {
        self.get.register(route, action);
    }

    pub fn register_post_action(
        &mut self,
        route: &str,
        action: Arc<dyn PostAction + Send + Sync + 'static>,
    ) {
        self.post.register(route, action);
    }

    pub fn register_put_action(
        &mut self,
        route: &str,
        action: Arc<dyn PutAction + Send + Sync + 'static>,
    ) {
        self.put.register(route, action);
    }

    pub fn register_delete_action(
        &mut self,
        route: &str,
        action: Arc<dyn DeleteAction + Send + Sync + 'static>,
    ) {
        self.delete.register(route, action);
    }
}

#[async_trait]
impl HttpServerMiddleware for ControllersMiddleware {
    async fn handle_request(&self, ctx: HttpContext) -> Result<MiddleWareResult, HttpFailResult> {
        let ref method = *ctx.get_method();
        match method {
            &Method::GET => {
                return self.get.handle_request(ctx).await;
            }
            &Method::POST => {
                return self.post.handle_request(ctx).await;
            }
            &Method::PUT => {
                return self.put.handle_request(ctx).await;
            }
            &Method::DELETE => {
                return self.delete.handle_request(ctx).await;
            }
            _ => {}
        }

        return Ok(MiddleWareResult::Next(ctx));
    }
}
