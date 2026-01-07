# HTTP Actions Design Pattern

This document describes the HTTP action architecture used in this project. Follow this pattern when creating new HTTP endpoints or building similar projects.

## Overview

HTTP actions are organized using a controller-based architecture where each action is:
- A self-contained struct with its own route, input model, and handler
- Registered through a centralized builder
- Automatically documented via Swagger/OpenAPI through macro annotations
- Separated from business logic (which lives in `scripts/`)

## Architecture Components

### 1. Directory Structure

```
src/
├── http/
│   ├── mod.rs                 # HTTP module exports
│   ├── builder.rs             # Controller registration
│   ├── start_up.rs            # HTTP server initialization
│   ├── errors.rs              # HTTP error types (if needed)
│   └── controllers/
│       ├── mod.rs             # Controller module exports
│       └── {controller_group}/
│           ├── mod.rs         # Group module exports
│           └── {action_name}_action.rs  # Individual action
├── scripts/                   # Business logic (called by actions)
└── app/
    └── app_ctx.rs             # Application context
```

### 2. Action Structure

Each HTTP action follows this pattern:

```rust
use std::sync::Arc;
use my_http_server::macros::*;
use my_http_server::*;
use crate::app::AppContext;

#[http_route(
    method: "GET" | "POST" | "PUT" | "DELETE" | "OPTIONS",
    route: "/api/{controller}/v1/{action-name}",
    deprecated_routes: ["/api/old-route"],  // Optional: legacy routes that still work
    summary: "Brief summary",
    description: "Detailed description",
    controller: "ControllerName",
    input_data: "InputModelName",
    authorized: Yes | No | YesWithClaims(["claim1", "claim2"]),  // Optional: authorization config
    result: [
        {status_code: 200, description: "Success description", model: "OptionalModel"},
        {status_code: 404, description: "Not found description"},
        {status_code: 500, description: "Error description"},
    ]
)]
pub struct ActionName {
    _app: Arc<AppContext>,
}

impl ActionName {
    pub fn new(app: Arc<AppContext>) -> Self {
        Self { _app: app }
    }
}

async fn handle_request(
    _action: &ActionName,
    input_data: InputModelName,
    _ctx: &HttpContext,
) -> Result<HttpOkResult, HttpFailResult> {
    // Call business logic from scripts/
    let result = crate::scripts::business_function(input_data.field).await;

    match result {
        Ok(output) => {
            // Return success response
            HttpOutput::as_json(output_model).into_ok_result(true).into()
            // OR for text:
            // HttpOutput::as_text(output).into_ok_result(true).into()
        }
        Err(error) => {
            // Handle different error types
            if error.contains("not found") {
                return HttpFailResult::as_not_found(error, false).into_err();
            }
            return HttpFailResult::as_fatal_error(error).into_err();
        }
    }
}
```

### 3. Input Models

Input models use the `MyHttpInput` derive macro and specify where data comes from. You can mix different input sources in a single model.

**Available Input Sources:**

1. **Query Parameters** (`#[http_query]`) - For GET requests and URL query strings
2. **Path Parameters** (`#[http_path]`) - For route path variables like `/api/users/{id}`
3. **HTTP Headers** (`#[http_header]`) - For reading HTTP headers
4. **Body Data** (`#[http_body]`) - For JSON body in POST/PUT requests
5. **Form Data** (`#[http_form_data]`) - For multipart/form-data requests
6. **Raw Body** (`#[http_body_raw]`) - For raw body content (only one field allowed)

**For POST/PUT requests (body data):**
```rust
#[derive(MyHttpInput)]
pub struct AddDomainInputModel {
    #[http_body(name = "domain", description = "Domain name to add certificate for")]
    pub domain: String,

    #[http_body(name = "email", description = "Email address for certificate registration")]
    pub email: String,
}
```

**For GET requests (query parameters):**
```rust
#[derive(MyHttpInput)]
pub struct GetCertInfoInputModel {
    #[http_query(name = "domain", description = "Domain name")]
    pub domain: String,
}
```

**Path Parameters:**
```rust
#[derive(MyHttpInput)]
pub struct GetUserInputModel {
    #[http_path(name = "id", description = "User ID")]
    pub id: String,
    
    #[http_query(name = "include_details", description = "Include user details", default = false)]
    pub include_details: bool,
}
```

**HTTP Headers:**
```rust
#[derive(MyHttpInput)]
pub struct ApiKeyInputModel {
    #[http_header(name = "X-API-Key", description = "API key for authentication")]
    pub api_key: String,
    
    #[http_header(name = "X-Request-ID", description = "Request ID for tracking", default = "")]
    pub request_id: Option<String>,
}
```

**Form Data (multipart/form-data):**
```rust
#[derive(MyHttpInput)]
pub struct UploadFileInputModel {
    #[http_form_data(name = "file", description = "File to upload")]
    pub file: Vec<u8>,  // File content
    
    #[http_form_data(name = "description", description = "File description")]
    pub description: String,
}
```

**Raw Body:**
```rust
#[derive(MyHttpInput)]
pub struct RawDataInputModel {
    #[http_body_raw(description = "Raw request body")]
    pub content: Vec<u8>,
}
```

**Field Options:**

All input field attributes support these optional parameters:
- `name` - Parameter name (defaults to field name if not specified)
- `description` - Description for Swagger documentation
- `default` - Default value if parameter is missing (e.g., `default = "value"`, `default = 0`, `default = false`)
- `validator` - Custom validator function name (must be in scope)
- `to_lowercase` - Convert value to lowercase before parsing
- `to_uppercase` - Convert value to uppercase before parsing
- `trim` - Trim whitespace before parsing
- `print_request_to_console` - Debug flag to print request details

**Optional Fields:**

Fields can be optional by using `Option<T>`:
```rust
#[derive(MyHttpInput)]
pub struct SearchInputModel {
    #[http_query(name = "query", description = "Search query")]
    pub query: String,
    
    #[http_query(name = "limit", description = "Result limit", default = 10)]
    pub limit: Option<u32>,  // Optional field
}
```

**Note:** You cannot mix `http_body`, `http_form_data`, and `http_body_raw` in the same model - only one body type is allowed per input model.

**Note on Field Transformations:** The `to_lowercase` and `to_uppercase` attributes work only with `String` types, not with other types like `Option<String>` or numeric types.

### 4. Output Models

Output models use `Serialize`, `Deserialize`, and `MyHttpObjectStructure`:

```rust
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, MyHttpObjectStructure)]
pub struct CertificateInfoHttpModel {
    pub cn: String,
    pub expires: String,
}
```

### 5. Response Types

The framework supports multiple response types through `HttpOutput`:

**JSON Response:**
```rust
HttpOutput::as_json(result_model).into_ok_result(true).into()
```

**Text Response:**
```rust
HttpOutput::as_text(output_string).into_ok_result(true).into()
```

**HTML Response:**
```rust
HttpOutput::as_html(html_content).into_ok_result(true).into()
```

**YAML Response:**
```rust
HttpOutput::as_yaml(result_model).into_ok_result(true).into()
```

**Empty Response (204 No Content):**
```rust
HttpOutput::Empty.into_ok_result(true).into()
```

**File Download:**
```rust
HttpOutput::as_file(
    "filename.txt".to_string(),
    file_content_bytes
).into_ok_result(true).into()
```

**Redirect Response:**
```rust
// Permanent redirect (301)
HttpOutput::as_redirect("https://example.com/new-url".to_string(), true)
    .into_ok_result(true).into()

// Temporary redirect (302)
HttpOutput::as_redirect("https://example.com/temp-url".to_string(), false)
    .into_ok_result(true).into()
```

**Custom Status Code and Headers:**
```rust
HttpOutput::from_builder()
    .with_status_code(201)
    .with_header("Location", "/api/resource/123")
    .with_content_type(WebContentType::Json)
    .with_cookie(cookie)
    .with_content(json_bytes)
    .build()
    .into_ok_result(true).into()
```

**Streaming Response:**
```rust
let (output, producer) = HttpOutput::as_stream(100);
// Send output in handle_request
// Use producer to send chunks asynchronously
output.into_ok_result(true).into()
```

### 6. Error Handling

Use appropriate error types based on the failure:

```rust
// Not Found (404)
HttpFailResult::as_not_found(error_message, false).into_err()

// Fatal Error (500)
HttpFailResult::as_fatal_error(error_message).into_err()

// Bad Request (400)
HttpFailResult::as_bad_request(error_message, false).into_err()

// Unauthorized (401) - Authentication required
HttpFailResult::as_unauthorized(Some("Authentication required")).into_err()

// Forbidden (403) - Not authorized
HttpFailResult::as_forbidden(error_message, false).into_err()

// Conflict (409)
HttpFailResult::as_conflict(error_message, false).into_err()

// Unprocessable Entity (422)
HttpFailResult::as_unprocessable_entity(error_message, false).into_err()
```

**Error Parameters:**

Most error methods take two parameters:
1. `error_message` - The error message to return
2. `write_log` - Boolean indicating whether to write to log (usually `false` for client errors, `true` for server errors)

**Custom Error with Status Code:**
```rust
HttpFailResult::new(
    HttpOutput::as_text("Custom error message"),
    false,  // write_log
    false   // write_telemetry
).into_err()
```

### 7. Controller Registration

Actions are registered in `src/http/builder.rs`:

```rust
use std::sync::Arc;
use my_http_server::controllers::{
    ControllersMiddleware, 
    ControllersAuthorization, 
    RequiredClaims,
    AuthErrorFactory
};
use crate::app::AppContext;

pub fn build_controllers(app: &Arc<AppContext>) -> ControllersMiddleware {
    // Create middleware with optional authorization
    let authorization = ControllersAuthorization::BearerAuthentication {
        global: true,  // Enable global authorization
        global_claims: RequiredClaims::from_slice_of_str(&["admin", "user"]),
    };
    
    let auth_error_factory: Option<Arc<dyn AuthErrorFactory + Send + Sync>> = 
        Some(Arc::new(crate::http::MyAuthErrorFactory::new()));
    
    let mut result = ControllersMiddleware::new(
        Some(authorization),  // Global authorization config
        auth_error_factory    // Custom error factory for auth failures
    );

    // Register POST actions
    result.register_post_action(Arc::new(
        crate::http::controllers::controller_group::ActionName::new(app.clone()),
    ));

    // Register GET actions
    result.register_get_action(Arc::new(
        crate::http::controllers::controller_group::ActionName::new(app.clone()),
    ));

    // Register PUT actions
    result.register_put_action(Arc::new(
        crate::http::controllers::controller_group::UpdateAction::new(app.clone()),
    ));

    // Register DELETE actions
    result.register_delete_action(Arc::new(
        crate::http::controllers::controller_group::DeleteAction::new(app.clone()),
    ));

    // Register OPTIONS actions (for CORS preflight)
    result.register_options_action(Arc::new(
        crate::http::controllers::controller_group::OptionsAction::new(app.clone()),
    ));

    result
}
```

**Authorization Types:**

The framework supports three authorization types:
- `BasicAuthentication` - HTTP Basic Auth
- `ApiKeys` - API key-based authentication
- `BearerAuthentication` - Bearer token (JWT) authentication

**Authorization Levels:**

In the `http_route` macro, you can specify:
- `authorized: Yes` - Requires authentication (uses global claims)
- `authorized: No` - No authentication required (public endpoint)
- `authorized: YesWithClaims(["claim1", "claim2"])` - Requires specific claims
- Omit `authorized` - Uses global authorization setting

**Deprecated Routes:**

Actions can support deprecated routes for backward compatibility:
```rust
#[http_route(
    method: "GET",
    route: "/api/v2/users/{id}",
    deprecated_routes: ["/api/v1/users/{id}", "/api/users/{id}"],
    // ... other parameters
)]
```
All deprecated routes will still work but may be marked as deprecated in Swagger documentation.

### 8. Module Organization

**Controller group module (`controllers/{group}/mod.rs`):**
```rust
pub mod action_name_action;
pub use action_name_action::*;
```

**Main controllers module (`controllers/mod.rs`):**
```rust
pub mod controller_group;
```

### 9. Server Startup

HTTP server is initialized in `src/http/start_up.rs`:

```rust
use std::{net::SocketAddr, sync::Arc};
use my_http_server::controllers::swagger::SwaggerMiddleware;
use my_http_server::MyHttpServer;
use crate::app::AppContext;

pub fn start(app: &Arc<AppContext>) {
    let mut http_server = MyHttpServer::new(SocketAddr::from(([0, 0, 0, 0], 8000)));

    let controllers = Arc::new(super::builder::build_controllers(&app));

    let swagger_middleware = SwaggerMiddleware::new(
        controllers.clone(),
        crate::app::APP_NAME.to_string(),
        crate::app::APP_VERSION.to_string(),
    );

    http_server.add_middleware(Arc::new(swagger_middleware));
    http_server.add_middleware(controllers);
    http_server.start(app.app_states.clone(), my_logger::LOGGER.clone());
}
```

## Design Principles

1. **Separation of Concerns**: HTTP actions are thin wrappers that delegate to business logic in `scripts/`
2. **Type Safety**: Use strongly-typed input/output models
3. **Documentation**: All routes are auto-documented via `http_route` macro
4. **Consistency**: Follow the same pattern for all actions
5. **Error Handling**: Use appropriate HTTP status codes and error types
6. **Modularity**: Group related actions in controller modules

## Creating a New Action

1. **Create the action file**: `src/http/controllers/{group}/{action_name}_action.rs`
2. **Define the action struct** with `#[http_route]` macro
3. **Create input model** with `#[derive(MyHttpInput)]`
4. **Create output model** (if returning JSON) with `Serialize`, `Deserialize`, `MyHttpObjectStructure`
5. **Implement `handle_request`** function that calls business logic
6. **Export in module**: Add to `{group}/mod.rs`
7. **Register in builder**: Add registration call in `builder.rs`

## Example: Complete Action

See `src/http/controllers/certbot/add_domain_action.rs` for a complete POST action example.

See `src/http/controllers/certificates/get_cert_info_action.rs` for a complete GET action example.

## Dependencies

This pattern requires:
- `my_http_server` crate with macros support
- `serde` for serialization
- `tokio` for async runtime
- Application context (`AppContext`) for shared state

## Advanced Features

### Route Path Parameters

Routes can include path parameters using `{param_name}` syntax:
```rust
#[http_route(
    method: "GET",
    route: "/api/users/{userId}/posts/{postId}",
    // ...
)]
```

The corresponding input model must have matching `#[http_path]` fields:
```rust
#[derive(MyHttpInput)]
pub struct GetPostInputModel {
    #[http_path(name = "userId", description = "User ID")]
    pub user_id: String,
    
    #[http_path(name = "postId", description = "Post ID")]
    pub post_id: String,
}
```

### Model Routes

Input models can define alternative route patterns through the `get_model_routes()` function (automatically generated). This allows the same action to handle multiple route patterns that map to the same input model structure.

### Field Validation

You can add custom validators to input fields. Validators can access the HTTP context for more complex validation:

```rust
#[derive(MyHttpInput)]
pub struct CreateUserInputModel {
    #[http_body(
        name = "email", 
        description = "Email address",
        validator = "validate_email"
    )]
    pub email: String,
}

// Simple validator (value only)
fn validate_email(value: &str) -> Result<(), String> {
    if value.contains('@') {
        Ok(())
    } else {
        Err("Invalid email format".to_string())
    }
}

// Validator with HTTP context access
fn validate_email_with_context(ctx: &HttpContext, value: &str) -> Result<(), HttpFailResult> {
    // Can access request headers, path, etc. from ctx
    if value.contains('@') {
        Ok(())
    } else {
        Err(HttpFailResult::as_validation_error(
            "Invalid email format".to_string()
        ))
    }
}
```

**Validator Signatures:**
- Simple: `fn validator_name(value: &str) -> Result<(), String>`
- With context: `fn validator_name(ctx: &HttpContext, value: &str) -> Result<(), HttpFailResult>`

### Debugging Input Models

Add `#[debug]` attribute to a field or use `print_request_to_console` flag to debug request parsing:
```rust
#[derive(MyHttpInput)]
pub struct DebugInputModel {
    #[http_query(
        name = "test",
        description = "Test parameter",
        print_request_to_console
    )]
    pub test: String,
}
```

### OPTIONS Method

The framework supports OPTIONS method for CORS preflight requests. Register OPTIONS actions the same way as other HTTP methods:
```rust
#[http_route(
    method: "OPTIONS",
    route: "/api/cors-endpoint",
    // ...
)]
```

### Enums as Input Types

The framework supports string and integer enums for input models using `MyHttpStringEnum` and `MyHttpIntegerEnum`:

**String Enum:**
```rust
#[derive(Clone, Copy, MyHttpStringEnum)]
pub enum DataSynchronizationPeriod {
    #[http_enum_case(id = "0", value = "i", description = "Immediately Persist")]
    Immediately,
    
    #[http_enum_case(id = "1", value = "1", description = "Persist during 1 sec")]
    Sec1,
    
    #[http_enum_case(id = "5", value = "5", description = "Persist during 5 sec", default)]
    Sec5,
    
    #[http_enum_case(id = "15", value = "15", description = "Persist during 15 sec")]
    Sec15,
}

#[derive(MyHttpInput)]
pub struct SyncInputModel {
    #[http_query(name = "syncPeriod", description = "Synchronization period", default = "Sec5")]
    pub sync_period: DataSynchronizationPeriod,
}
```

**Integer Enum:**
```rust
#[derive(Clone, Copy, MyHttpIntegerEnum)]
pub enum StatusCode {
    #[http_enum_case(id = "200", description = "OK")]
    Ok,
    
    #[http_enum_case(id = "404", description = "Not Found")]
    NotFound,
}
```

**Enum Case Attributes:**
- `id` - Numeric identifier for the enum case (required)
- `value` - String value used in HTTP requests (optional, defaults to variant name)
- `description` - Description for Swagger documentation (required)
- `default` - Marks this case as the default value (optional)

### Custom HttpInputFields

You can create custom input field types based on `String` with additional validation and processing. This is useful for fields like passwords, emails, or other types that need special handling:

```rust
#[http_input_field(open_api_type: "Password")]
pub struct PasswordField(String);

fn process_value(src: &str) -> Result<rust_extensions::StrOrString, HttpFailResult> {
    // Password validation
    if src.len() < 8 {
        return Err(HttpFailResult::as_validation_error(
            "Password must be at least 8 characters long".to_string(),
        ));
    }
    
    let src = src.trim();
    let src = src.to_lowercase();
    Ok(rust_extensions::StrOrString::create_as_string(src))
}
```

**Usage in Input Models:**
```rust
#[derive(MyHttpInput)]
pub struct AuthenticateInputModel {
    #[http_form_data(description = "Email of user")]
    pub email: String,
    
    #[http_form_data(description = "Password of user")]
    pub password: PasswordField,  // Custom field type
}
```

**OpenAPI Types:**
- `String` (default)
- `Password` - Renders as password input in Swagger UI

### Working with Cookies

You can set cookies in responses using `CookieJar`:

```rust
use my_http_server::cookies::{Cookie, CookieJar};

async fn handle_request(
    _action: &ActionName,
    input_data: InputModelName,
    _ctx: &HttpContext,
) -> Result<HttpOkResult, HttpFailResult> {
    let mut cookies = CookieJar::new();
    
    // Set cookie with options
    cookies.set_cookie(
        Cookie::new("SessionId", "abc123")
            .set_domain("/")
            .set_max_age(24 * 60 * 60),  // 24 hours
    );
    
    // Simple cookie setting
    cookies.set_cookie(("Test2".to_string(), "Value".to_string()));
    cookies.set_cookie(("Test3", "Value".to_string()));
    
    HttpOutput::from_builder()
        .with_cookies(cookies)
        .with_content_type(WebContentType::Json)
        .with_content(json_bytes)
        .build()
        .into_ok_result(true).into()
}
```

## Notes

- Actions receive `Arc<AppContext>` for shared application state
- The `_app` field is prefixed with `_` if not directly used in the handler
- Swagger documentation is automatically generated from `http_route` annotations
- Business logic should be implemented in `scripts/` module, not directly in actions
- Path parameters in routes must match `#[http_path]` fields in input models
- Only one body type (`http_body`, `http_form_data`, or `http_body_raw`) can be used per input model
- Headers are case-insensitive when reading
- Optional fields use `Option<T>` type
- Default values can be specified for any input field attribute
- Field transformations (`to_lowercase`, `to_uppercase`, `trim`) are applied before parsing
- `to_lowercase` and `to_uppercase` attributes work only with `String` types
- Enums must have at least one case marked with `default` if used with default values
- Custom input fields must implement `TryInto` trait for conversion from HTTP parameter types

## References

- [GitHub Wiki](https://github.com/MyJetTools/my-http-server/wiki) - Official documentation and examples