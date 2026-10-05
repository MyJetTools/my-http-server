# HTTP Actions Design Pattern

This document describes the HTTP action architecture used in this project. Follow this pattern when creating new HTTP endpoints or building similar projects.

This page is the shape every action has: the directory structure, the action struct with `#[http_route]` and `handle_request`, the steps of creating an action and the rules that hold everywhere. Input and output models, responses, errors, registration, a complete example and the less common features are topics, listed at the end. Read a topic: `resource://http-actions-design-guide/{topic}`, or `get_http_actions_design_guide` with `topic`.

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
├── http_server/
│   ├── mod.rs                      # HTTP module exports
│   ├── build_controllers.rs        # Controller registration (fn build_controllers)
│   ├── start_up.rs                 # Creates and starts MyHttpServer (fn start)
│   ├── errors.rs                   # HTTP error types + From<DomainError> (if needed)
│   └── controllers/
│       ├── mod.rs                  # Controller module exports
│       └── {controller_group}/
│           ├── mod.rs              # Group module exports
│           └── {action_name}_action.rs  # Individual action
├── scripts/                        # Business logic (called by actions)
└── app/
    └── app_ctx.rs                  # Application context
```

`AppContext` holds the application states (`Arc<AppStates>` from rust-extensions): `MyHttpServer::start` takes them and stops accepting connections once they say the application is shutting down.

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

**Important Notes:**
- All fields in the `http_route` macro must be separated by commas (including the last field before `summary`, `description`, etc.)
- When using `model:` in the `result:` field, the model **MUST** derive `MyHttpObjectStructure` (see topic [`output-models`](output-models.md))

## Design Principles

1. **Separation of Concerns**: HTTP actions are thin wrappers that delegate to business logic in `scripts/`
2. **Type Safety**: Use strongly-typed input/output models
3. **Documentation**: All routes are auto-documented via `http_route` macro
4. **Consistency**: Follow the same pattern for all actions
5. **Error Handling**: Use appropriate HTTP status codes and error types
6. **Modularity**: Group related actions in controller modules

## Creating a New Action

1. **Create the action file**: `src/http_server/controllers/{group}/{action_name}_action.rs`
2. **Define the action struct** with `#[http_route]` macro
3. **Create input model** with `#[derive(MyHttpInput)]`
4. **Create output model** (if returning JSON):
   - If using `model:` in result: `Serialize`, `Deserialize`, `MyHttpObjectStructure`
   - If NOT using `model:` in result: `Serialize` (or `Serialize, Deserialize` if needed)
5. **Implement `handle_request`** function that calls business logic
6. **Export in module**: Add to `{group}/mod.rs`
7. **Register in build_controllers**: Add `http_server_builder.register_*_action(...)` call in `build_controllers.rs`

## Dependencies

This pattern requires:
- `my_http_server` crate with macros support
- `my-http-utils` as a direct dependency of every crate which derives `MyHttpInput` or `MyHttpObjectStructure`: the derives expand to `my_http_utils::…` paths, and the re-export through `my_http_server::macros` does not make that name resolve in your crate
- `serde` for serialization
- `tokio` for async runtime
- Application context (`AppContext`) for shared state

## Notes

- Actions receive `Arc<AppContext>` for shared application state
- The `_app` field is prefixed with `_` if not directly used in the handler
- Swagger documentation is automatically generated from `http_route` annotations
- Business logic should be implemented in `scripts/` module, not directly in actions
- Path parameters in routes must match `#[http_path]` fields in input models
- Only one body type (`http_body`, `http_form_data`, `http_body_raw` or `http_body_as_stream`) can be used per input model
- Headers are case-insensitive when reading
- Optional fields use `Option<T>` type
- Default values can be specified for any input field attribute
- Field transformations (`to_lowercase`, `to_uppercase`, `trim`) are applied before parsing
- `to_lowercase` and `to_uppercase` attributes work only with `String` types
- Enums must have at least one case marked with `default` if used with default values
- Custom input fields must implement `TryInto` trait for conversion from HTTP parameter types
- **When using `model:` in result field, always derive `MyHttpObjectStructure`** - this is a common mistake that causes compilation errors

## References

- [GitHub Wiki](https://github.com/MyJetTools/my-http-server/wiki) - Official documentation and examples

## Topics

| Topic | What is inside |
| --- | --- |
| [`input-models`](input-models.md) | `MyHttpInput` and every input source: query, path, headers, JSON body, form data, `FileContent`, raw body, `RawDataTyped<T>`; field options, optional fields, parameter-less actions |
| [`streamed-body`](streamed-body.md) | `#[http_body_as_stream]`: reading an upload in chunks — memory, truncated uploads, `read_to_end`, the body read timeout, taking the body once |
| [`output-models`](output-models.md) | Response models: when `MyHttpObjectStructure` is required (`model:` in `result:`), the compile error without it |
| [`shared-models`](shared-models.md) | Models of an API a UI consumes live in a wasm-clean `rest-api-shared` crate with a `server` feature |
| [`responses`](responses.md) | `HttpOutput`: JSON, text, HTML, YAML, empty, file, redirect, custom status and headers, streaming, gzip compression |
| [`errors`](errors.md) | `HttpFailResult` helpers and their log / telemetry flags, custom statuses; centralized domain → HTTP error conversions in `errors.rs` |
| [`registration-and-startup`](registration-and-startup.md) | `build_controllers` on a `ControllersMiddleware`, authorization levels, deprecated routes, module exports, starting `MyHttpServer` with Swagger |
| [`example`](example.md) | Two complete action files: a GET with a query and a JSON response, a POST with a JSON body and an empty response |
| [`troubleshooting`](troubleshooting.md) | Common compilation errors of `http_route` and response models |
| [`routes`](routes.md) | Path parameters in routes, model routes, the OPTIONS method |
| [`field-validation`](field-validation.md) | Field validators (with and without `HttpContext`), custom `#[http_input_field]` types, debugging input parsing |
| [`enums`](enums.md) | `MyHttpStringEnum` / `MyHttpIntegerEnum` as input types and their case attributes |
| [`client-ip`](client-ip.md) | The client ip from `get_ip()` — never parse `X-Forwarded-For` by hand |
| [`cookies`](cookies.md) | Setting cookies on the response builder |
