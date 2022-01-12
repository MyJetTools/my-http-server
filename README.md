# my-http-server


Cargo.toml

````Toml
tokio = { version = "*", features = ["full"] }
tokio-util = "*"
hyper = { version = "*", features = ["full"] }
my-http-server = { branch = "main", git = "https://github.com/MyJetTools/my-http-server.git" }
````



Hyper based Http Server



````Rust

use my_http_server::middlewares::swagger::SwaggerMiddleware;
use my_http_server::MyHttpServer;

#[tokio::main]
async fn main() {

    let mut http_server: MyHttpServer = MyHttpServer::new(SocketAddr::from(([0, 0, 0, 0], 8080)));
    
    let controllers = Arc::new(crate::http::controllers::builder::build(app.clone()));
    
        http_server.add_middleware(Arc::new(SwaggerMiddleware::new(
        controllers.clone(),
        "MyService".to_string(),
        crate::app::APP_VERSION.to_string(),
    )));
    
    http_server.add_middleware(controllers);
    
    http_server.start(app.clone());

}
````


To Create a controller - just one of the traits has to be implemented

````Rust
pub struct MyController {
    app: Arc<AppContext>,
}

impl MyController {
    pub fn new(app: Arc<AppContext>) -> Self {
        Self { app }
    }
}


//We are implementing a Post action
#[async_trait]
impl PostAction for MyController {

    //Swagger description
    fn get_description(&self) -> Option<HttpActionDescription> {
        HttpActionDescription {
            name: "Name of the action",
            description: "Description of the Action",
            out_content_type: WebContentType::Json,
            input_params: Some(vec![
                HttpInputParameter {
                    name: "name".to_string(),
                    param_type: HttpParameterType::String,
                    description: "Name of client application".to_string(),
                    source: HttpParameterInputSource::Path,
                    required: true,
                },
                HttpInputParameter {
                    name: "version".to_string(),
                    param_type: HttpParameterType::String,
                    description: "Version of client application".to_string(),
                    source: HttpParameterInputSource::Query,
                    required: true,
                },
            ]),
        }
        .into()
    }
    
    async fn handle_request(&self, ctx: HttpContext) -> Result<HttpOkResult, HttpFailResult> {
        let query = ctx.get_query_string()?;

        let app_name = ctx.get_value_from_path_optional("name")?;
        let app_version = query.get_required_string_parameter("version")?;
        
        /////... Logic here
        
        let result = JsonResult { session: id };

        HttpOkResult::create_json_response(result).into()
    }
    

}
````


//We are registering a controller

````Rust
pub fn build(app: Arc<AppContext>) -> ControllersMiddleware {
    let mut controllers = ControllersMiddleware::new();
 
    let my_controller = Arc::new(super::topics::MyController::new(app.clone()));

    controllers.register_get_action("/MyController/{name}", my_controller);
    
    controllers   
}

````

