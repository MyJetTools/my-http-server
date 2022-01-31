use std::sync::Arc;

use async_trait::async_trait;
use crate::{HttpServerMiddleware, HttpContext, HttpOkResult, MiddleWareResult, HttpFailResult};

use super::is_alive_model;
use super::dependencies_model::Dependencies;

include!(concat!(env!("OUT_DIR"), "/repo_deps.rs"));

const API_URL_DEPENDENCIES: &str = "/api/dependencies";
const API_URL_ISALIVE: &str = "/api/isalive";

pub struct HealthcheckMiddleware {
    deps: Arc<Dependencies>
}

impl HealthcheckMiddleware {
    pub fn new() -> Self {
        let hs = return_project_deps();
        let deps = Dependencies::from(hs);

        Self {
            deps: Arc::new(deps)
        }
    }

    pub fn from_hash_map(dependencies: Arc<Dependencies>) -> Self {
        Self {
            deps: dependencies
        }
    }
}

#[async_trait]
impl HttpServerMiddleware for HealthcheckMiddleware {
    async fn handle_request(&self, ctx: HttpContext) -> Result<MiddleWareResult, HttpFailResult> {
        // PATH
        let path = ctx.get_path_lower_case();

        // ROUTE
        match path {
            API_URL_DEPENDENCIES => return self.get_dependencies_route(),
            API_URL_ISALIVE => return self.check_is_alive_route(),
            _ => return Ok(MiddleWareResult::Next(ctx))
        }
    }    
}

impl HealthcheckMiddleware {
    fn check_is_alive_route(&self) -> Result<MiddleWareResult, HttpFailResult>{
        let model = is_alive_model::read();
            let result = HttpOkResult::create_json_response(model);
            return Ok(MiddleWareResult::Ok(result));
    }

    fn get_dependencies_route(&self) -> Result<MiddleWareResult, HttpFailResult> {
        let result = HttpOkResult::create_json_response(self.deps.as_ref());
        return Ok(MiddleWareResult::Ok(result));
    }
}