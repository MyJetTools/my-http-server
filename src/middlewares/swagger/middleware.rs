use std::sync::Arc;

use crate::{
    HttpContext, HttpFailResult, HttpOkResult, HttpServerMiddleware, MiddleWareResult,
    WebContentType,
};
use async_trait::async_trait;

use super::{super::controllers::ControllersMiddleware, swagger_model::SwaggerJsonModel};

pub struct SwaggerMiddleware {
    controllers: Arc<ControllersMiddleware>,
    title: String,
    version: String,
}

impl SwaggerMiddleware {
    pub fn new(controllers: Arc<ControllersMiddleware>, title: String, version: String) -> Self {
        Self {
            controllers,
            title,
            version,
        }
    }
}

#[async_trait]
impl HttpServerMiddleware for SwaggerMiddleware {
    async fn handle_request(&self, ctx: HttpContext) -> Result<MiddleWareResult, HttpFailResult> {
        let path = ctx.get_path_lower_case();

        if !path.starts_with("/swagger") {
            return Ok(MiddleWareResult::Next(ctx));
        }

        if path == "/swagger/index.html" {
            let result = HttpOkResult::Content {
                content_type: Some(WebContentType::Html),
                content: super::resources::INDEX_PAGE.to_vec(),
            };
            return Ok(MiddleWareResult::Ok(result));
        }

        if path == "/swagger/swagger-ui.css" {
            let result = HttpOkResult::Content {
                content_type: Some(WebContentType::Css),
                content: super::resources::SWAGGER_UI_CSS.to_vec(),
            };
            return Ok(MiddleWareResult::Ok(result));
        }

        if path == "/swagger/swagger-ui-bundle.js" {
            let result = HttpOkResult::Content {
                content_type: Some(WebContentType::JavaScript),
                content: super::resources::SWAGGER_UI_BUNDLE_JS.to_vec(),
            };
            return Ok(MiddleWareResult::Ok(result));
        }

        if path == "/swagger/swagger-ui-standalone-preset.js" {
            let result = HttpOkResult::Content {
                content_type: Some(WebContentType::JavaScript),
                content: super::resources::SWAGGER_UI_STANDALONE_PRESET_JS.to_vec(),
            };
            return Ok(MiddleWareResult::Ok(result));
        }

        if path == "/swagger/favicon-32x32.png" {
            let result = HttpOkResult::Content {
                content_type: Some(WebContentType::Png),
                content: super::resources::FAVICON_32.to_vec(),
            };
            return Ok(MiddleWareResult::Ok(result));
        }

        if path == "/swagger/favicon-16x16.png" {
            let result = HttpOkResult::Content {
                content_type: Some(WebContentType::Png),
                content: super::resources::FAVICON_16.to_vec(),
            };
            return Ok(MiddleWareResult::Ok(result));
        }

        let scheme = ctx.get_scheme();

        let host = ctx.get_host();

        if path == "/swagger" {
            let new_url = format!("{}://{}/swagger/index.html", scheme, host);
            return Ok(MiddleWareResult::Ok(HttpOkResult::Redirect {
                url: new_url,
            }));
        }

        if path == "/swagger/v1/swagger.json" {
            let mut json_model = SwaggerJsonModel::new(
                self.title.clone(),
                self.version.clone(),
                host.to_string(),
                scheme.to_string(),
            );

            json_model.populate_operations(self.controllers.as_ref());

            let result = HttpOkResult::create_json_response(json_model);

            return Ok(MiddleWareResult::Ok(result));
            /*
            let mut placehloders = HashMap::new();

            placehloders.insert("SCHEME", ctx.get_scheme());

            placehloders.insert("HOST", host.to_string());
            placehloders.insert("VERSION", crate::app::APP_VERSION.to_string());

            let result = super::super::files::replace_placeholders(
                super::resources::swagger_json,
                &placehloders,
            );

            let result = HttpOkResult::Content {
                content_type: Some(WebContentType::Json),
                content: result,
            };

            return Ok(MiddleWareResult::Ok(result));
            */
        }

        let result = super::super::files::get(format!("./wwwroot{}", path).as_str()).await;

        match result {
            Ok(content) => {
                let result = HttpOkResult::Content {
                    content_type: None,
                    content,
                };
                return Ok(MiddleWareResult::Ok(result));
            }
            _ => {
                let new_url = format!("{}://{}/swagger/index.html", scheme, host);
                return Ok(MiddleWareResult::Ok(HttpOkResult::Redirect {
                    url: new_url,
                }));
            }
        }
    }
}
