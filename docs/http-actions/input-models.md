# Input Models

Input models use the `MyHttpInput` derive macro and specify where data comes from. You can mix different input sources in a single model.

**Available Input Sources:**

1. **Query Parameters** (`#[http_query]`) - For GET requests and URL query strings
2. **Path Parameters** (`#[http_path]`) - For route path variables like `/api/users/{id}`
3. **HTTP Headers** (`#[http_header]`) - For reading HTTP headers
4. **Body Data** (`#[http_body]`) - For JSON body in POST/PUT requests
5. **Form Data** (`#[http_form_data]`) - For multipart/form-data requests
6. **Raw Body** (`#[http_body_raw]`) - For raw body content (only one field allowed)
7. **Streamed Body** (`#[http_body_as_stream]`) - Reads the body in chunks instead of materializing it (uploads, proxying) — topic [`streamed-body`](streamed-body.md)

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

**File Uploads with FileContent:**

For file uploads, you can use `FileContent` to access file metadata (filename, content type) along with the content:

```rust
use my_http_server::FileContent;

#[derive(MyHttpInput)]
pub struct UploadFileWithMetadataInputModel {
    #[http_form_data(name = "file", description = "File to upload")]
    pub file: FileContent,  // Contains file_name, content_type, and content
    
    #[http_form_data(name = "description", description = "File description")]
    pub description: String,
}

async fn handle_request(
    _action: &ActionName,
    input_data: UploadFileWithMetadataInputModel,
    _ctx: &HttpContext,
) -> Result<HttpOkResult, HttpFailResult> {
    // Access file metadata
    let file_name = input_data.file.file_name;
    let content_type = input_data.file.content_type;
    let content = input_data.file.content;
    
    // Process the file...
    
    HttpOutput::as_json(result).into_ok_result(true).into()
}
```

**FileContent Structure:**
```rust
pub struct FileContent {
    pub content_type: String,  // MIME type (e.g., "image/png", "application/pdf")
    pub file_name: String,     // Original filename
    pub content: Vec<u8>,      // File content as bytes
}
```

**Note:** `FileContent` can only be used with `#[http_form_data]` attributes and only works with actual file uploads in multipart/form-data requests. It cannot be used with query parameters, headers, or JSON body data.

**Raw Body:**
```rust
#[derive(MyHttpInput)]
pub struct RawDataInputModel {
    #[http_body_raw(description = "Raw request body")]
    pub content: Vec<u8>,
}
```

**Typed raw body (`RawDataTyped<T>`) — deserialize with `?`:**

Instead of `Vec<u8>` you can type a `#[http_body_raw]` field as `RawDataTyped<T>`. It is built **infallibly** (it just holds the raw bytes) — `T` is parsed only when you call `.deserialize_json()`, which returns `Result<T, my_http_utils::http_input::HttpParseError>`.

`HttpFailResult` implements `From<HttpParseError>`, so that call ends with `?` right in the handler — a malformed body is turned into the correct validation-error `HttpFailResult` (status + text) for you, no `map_err`:

```rust
use my_http_server::RawDataTyped;

#[derive(MyHttpInput)]
pub struct CreateOrderInputModel {
    #[http_body_raw(description = "Order payload as JSON")]
    pub body: RawDataTyped<OrderPayload>,
}

async fn handle_request(
    _action: &ActionName,
    input_data: CreateOrderInputModel,
    _ctx: &HttpContext,
) -> Result<HttpOkResult, HttpFailResult> {
    // deserialize_json() -> Result<OrderPayload, HttpParseError>; `?` converts the error
    // into an HttpFailResult via `From<HttpParseError>`.
    let payload: OrderPayload = input_data.body.deserialize_json()?;
    // ... use payload ...
}
```

The same `From<HttpParseError> for HttpFailResult` conversion backs the generated `parse` in every action, so **any** `my_http_utils` call that returns `Result<_, HttpParseError>` (e.g. `HttpInputValue::deserialize_json` / `::parse`) can be finished with `?` inside a handler.

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

**Note:** You cannot mix `http_body`, `http_form_data`, `http_body_raw` and `http_body_as_stream` in the same model - only one body type is allowed per input model.

**How the body is read.** A `#[http_body]` / `#[http_form_data]` body is parsed as it comes off the wire rather than collected first. Of a JSON body only the members the model reads are kept; the rest is still read, and let go, before the action runs — so a body cut short never reaches it, and the connection can be used again. A url-encoded or multipart body can only be parsed whole and is collected first. A body that announced a `Content-Encoding`, or that a middleware has already read, is parsed from the materialized (decoded) bytes instead. `#[http_body_raw]` always gets the whole body. A malformed JSON body is answered `412` as soon as the broken part arrives.

**Note on Field Transformations:** The `to_lowercase` and `to_uppercase` attributes work only with `String` types, not with other types like `Option<String>` or numeric types.

## Parameter-less actions

When an action takes **no input data at all** (a `GET /ping`, a `POST /logout` that reads everything it needs from the auth context, etc.), do **not** declare an empty input model:

```rust
// ❌ Don't do this — an empty MyHttpInput model buys you nothing
#[derive(MyHttpInput)]
pub struct LogoutInputModel {}
```

Instead, drop the input entirely:

- **Remove `input_data:`** from the `#[http_route]` macro.
- **Remove the `input_data` parameter** from `handle_request`. The signature collapses to `(action: &XAction, ctx: &HttpContext) -> Result<HttpOkResult, HttpFailResult>`.

This is exactly the shape of the `ping` example — a `GET` with no input.

**Before / after:**

```rust
// Before — an empty model threaded through for nothing
#[http_route(
    method: "POST",
    route: "/api/auth/v1/logout",
    controller: "Auth",
    input_data: "LogoutInputModel",       // ← remove
    // ...
)]
pub struct LogoutAction { /* ... */ }

#[derive(MyHttpInput)]
pub struct LogoutInputModel {}             // ← delete

async fn handle_request(
    action: &LogoutAction,
    _input_data: LogoutInputModel,         // ← remove
    ctx: &HttpContext,
) -> Result<HttpOkResult, HttpFailResult> { /* ... */ }
```

```rust
// After — no input_data in the macro, no parameter in the handler
#[http_route(
    method: "POST",
    route: "/api/auth/v1/logout",
    controller: "Auth",
    // no input_data field at all
    // ...
)]
pub struct LogoutAction { /* ... */ }

async fn handle_request(
    action: &LogoutAction,
    ctx: &HttpContext,
) -> Result<HttpOkResult, HttpFailResult> { /* ... */ }
```
