use async_trait::async_trait;
use std::sync::Arc;

use crate::{
    request_flow::HttpServerRequestFlow, HttpContext, HttpFailResult, HttpOkResult,
    HttpServerMiddleware,
};
use hyper::Method;

use super::{
    actions::{DeleteAction, GetAction, PostAction, PutAction},
    documentation::data_types::HttpObjectStructure,
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
    pub http_objects: Vec<HttpObjectStructure>,
}

impl ControllersMiddleware {
    pub fn new() -> Self {
        Self {
            get: GetRoute::new(),
            post: PostRoute::new(),
            put: PutRoute::new(),
            delete: DeleteRoute::new(),
            http_objects: Vec::new(),
        }
    }

    pub fn register_get_action(&mut self, action: Arc<dyn GetAction + Send + Sync + 'static>) {
        self.get.register(action);
    }

    pub fn register_post_action(&mut self, action: Arc<dyn PostAction + Send + Sync + 'static>) {
        self.post.register(action);
    }

    pub fn register_put_action(&mut self, action: Arc<dyn PutAction + Send + Sync + 'static>) {
        self.put.register(action);
    }

    pub fn register_delete_action(
        &mut self,
        action: Arc<dyn DeleteAction + Send + Sync + 'static>,
    ) {
        self.delete.register(action);
    }

    pub fn list_of_get_route_actions<'s>(&'s self) -> Vec<&'s GetRouteAction> {
        let mut result = Vec::with_capacity(self.get.no_keys.len() + self.get.with_keys.len());

        result.extend(self.get.no_keys.values());
        result.extend(&self.get.with_keys);

        result
    }

    pub fn list_of_post_route_actions<'s>(&'s self) -> Vec<&'s PostRouteAction> {
        let mut result = Vec::with_capacity(self.post.no_keys.len() + self.post.with_keys.len());

        result.extend(self.post.no_keys.values());
        result.extend(&self.post.with_keys);

        result
    }

    pub fn list_of_put_route_actions<'s>(&'s self) -> Vec<&'s PutRouteAction> {
        let mut result = Vec::with_capacity(self.put.no_keys.len() + self.put.with_keys.len());

        result.extend(self.put.no_keys.values());
        result.extend(&self.put.with_keys);

        result
    }

    pub fn list_of_delete_route_actions<'s>(&'s self) -> Vec<&'s DeleteRouteAction> {
        let mut result =
            Vec::with_capacity(self.delete.no_keys.len() + self.delete.with_keys.len());

        result.extend(self.delete.no_keys.values());
        result.extend(&self.delete.with_keys);

        result
    }
}

#[async_trait]
impl HttpServerMiddleware for ControllersMiddleware {
    async fn handle_request(
        &self,
        ctx: &mut HttpContext,
        get_next: &mut HttpServerRequestFlow,
    ) -> Result<HttpOkResult, HttpFailResult> {
        let ref method = *ctx.request.get_method();
        match method {
            &Method::GET => {
                if let Some(result) = self.get.handle_request(ctx).await? {
                    return Ok(result);
                } else {
                    return get_next.next(ctx).await;
                }
            }
            &Method::POST => {
                if let Some(result) = self.post.handle_request(ctx).await? {
                    return Ok(result);
                } else {
                    return get_next.next(ctx).await;
                }
            }
            &Method::PUT => {
                if let Some(result) = self.put.handle_request(ctx).await? {
                    return Ok(result);
                } else {
                    return get_next.next(ctx).await;
                }
            }
            &Method::DELETE => {
                if let Some(result) = self.delete.handle_request(ctx).await? {
                    return Ok(result);
                } else {
                    return get_next.next(ctx).await;
                }
            }
            _ => {}
        }

        return get_next.next(ctx).await;
    }
}
