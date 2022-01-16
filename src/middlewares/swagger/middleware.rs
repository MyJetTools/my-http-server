use std::sync::Arc;

use crate::{
    HttpContext, HttpFailResult, HttpOkResult, HttpServerMiddleware, MiddleWareResult,
    WebContentType,
};
use async_trait::async_trait;
use tokio::sync::Mutex;

use super::{super::controllers::ControllersMiddleware, json_object_writer::JsonObjectWriter};

pub struct SwaggerMiddleware {
    controllers: Arc<ControllersMiddleware>,
    title: String,
    version: String,
    swagger_json_ok_result: Mutex<Option<HttpOkResult>>,
}

impl SwaggerMiddleware {
    pub fn new(controllers: Arc<ControllersMiddleware>, title: String, version: String) -> Self {
        Self {
            controllers,
            title,
            version,
            swagger_json_ok_result: Mutex::new(None),
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
            let mut write_access = self.swagger_json_ok_result.lock().await;
            if let Some(result) = &*write_access {
                return Ok(MiddleWareResult::Ok(result.clone()));
            }

            let mut json_object_writer = JsonObjectWriter::as_object();

            super::swagger_json::title::write(
                &mut json_object_writer,
                host,
                self.title.as_str(),
                self.version.as_str(),
            );
            json_object_writer.write_object("scheme", super::swagger_json::hosts::get(host));

            if let Some(definitions) =
                super::swagger_json::definitions::build(self.controllers.as_ref())
            {
                json_object_writer.write_object("definitions", definitions);
            }

            *write_access = Some(HttpOkResult::Content {
                content_type: Some(WebContentType::Json),
                content: json_object_writer.build(),
            });

            return Ok(MiddleWareResult::Ok(write_access.as_ref().unwrap().clone()));
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
