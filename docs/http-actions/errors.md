# Errors

Use appropriate error types based on the failure:

```rust
// Not Found (404) - the second argument says whether to write telemetry
HttpFailResult::as_not_found(error_message, false).into_err()

// Validation error (400)
HttpFailResult::as_validation_error(error_message).into_err()

// Unauthorized (401) - Authentication required
HttpFailResult::as_unauthorized(Some("Authentication required")).into_err()

// Forbidden (403) - Not authorized
HttpFailResult::as_forbidden(Some(error_message)).into_err()

// Unsupported Media Type (415)
HttpFailResult::as_not_supported_content_type(error_message).into_err()

// Fatal Error (500)
HttpFailResult::as_fatal_error(error_message).into_err()

// Any other status with a text body - e.g. Conflict (409), Unprocessable Entity (422)
HttpFailResult::from((409, "Order is already closed")).into_err()
```

There is no `as_bad_request`, `as_conflict` or `as_unprocessable_entity`: a 400 is
`as_validation_error`, and any other status goes through `HttpFailResult::from((status_code, text))`
(`text` is a `String` or a `&'static str`).

**Log and telemetry:**

Each helper decides on its own whether the failure is written to the log and to telemetry:

| Helper | Log | Telemetry |
|---|---|---|
| `as_not_found(text, write_telemetry)` | no | the second argument |
| `as_validation_error(text)` | no | yes |
| `as_unauthorized(text)` | no | no |
| `as_forbidden(text)` | no | yes |
| `as_not_supported_content_type(text)` | yes | yes |
| `as_fatal_error(text)` | yes | yes |
| `HttpFailResult::from((status_code, text))` | no | yes |

**Custom Error with Status Code:**

To choose the status code, the body and both flags yourself, build the output and hand it over:

```rust
HttpFailResult::new(
    HttpOutput::as_text("Custom error message")
        .set_status_code(418)
        .build(),
    false, // write_to_log
    false, // write_telemetry
)
.into_err()

// The same, straight from the builder
HttpOutput::as_text("Custom error message")
    .set_status_code(418)
    .into_err(false, false) // write_log, write_telemetry
```

## Centralized HTTP Error Handling and Conversions (Addendum)

### Goals
- Keep controllers thin: no inline `map_err`.
- Convert domain/script errors to `HttpFailResult` in one place.
- Build HTTP errors via `HttpOutput::from_builder()` + `HttpFailResult::new`.
- Controllers only orchestrate and use `?`.

### File Layout
- `src/http_server/errors.rs`: all HTTP error helpers and `From` conversions.
- `src/http_server/mod.rs`: export `errors` — `mod errors; pub use errors::*;`
- Controllers import helpers from `crate::http_server::errors`.

### Helpers (pattern)
```rust
fn text_error(status: u16, msg: impl Into<String>) -> HttpFailResult;
pub fn bad_request(msg: impl Into<String>) -> HttpFailResult;

pub fn compile_solidity_http(source: &str, contract: Option<&str>)
    -> Result<(String, String), HttpFailResult>;

pub fn encode_constructor_args_http(abi: &str, params: &[String])
    -> Result<String, HttpFailResult>;

pub async fn deploy_contract_http(
    app: &Arc<AppContext>,
    private_key: &str,
    bytecode: &str,
    constructor_args: Option<&str>,
    value_eth: Option<f64>,
) -> Result<(TxHash, Address), HttpFailResult>;

pub fn parse_private_key_http(hex: &str)
    -> Result<B256, HttpFailResult>;

pub fn signer_from_bytes(key: &B256)
    -> Result<PrivateKeySigner, HttpFailResult>;
```

#### Domain Error Conversions
Implement `From` for domain/script errors in `errors.rs`, for example:
```rust
impl From<SoliditySourceError> for HttpFailResult {
    match err {
        MissingSource => bad_request("Either solidity_url or solidity_code must be provided"),
        InvalidUrl(msg) => bad_request(format!("Invalid solidity_url: {}", msg)),
        DownloadFailed { status, message } =>
            text_error(status.unwrap_or(400), format!("Failed to download Solidity source: {}", message)),
        ReadFailed(msg) => bad_request(format!("Failed to read Solidity source: {}", msg)),
        Utf8Error(msg) => HttpFailResult::as_fatal_error(
            format!("Failed to decode Solidity source as UTF-8: {}", msg)
        ),
    }
}
```

### Controller Usage Pattern
- Import helpers from `http_server::errors`.
- Use `?` everywhere; avoid inline `map_err`.
- Example flow:
```rust
let src = fetch_solidity_source(...).await?; // uses From<SoliditySourceError>
let (bytecode, abi) = compile_solidity_http(&src, contract_name)?;
let encoded = encode_constructor_args_http(&abi, &params)?;
let (tx_hash, addr) = deploy_contract_http(app, pk, &bytecode, encoded.as_deref(), value).await?;
let signer = signer_from_bytes(&parse_private_key_http(pk)?)?;
Ok(HttpOutput::as_json(resp).into_ok_result(false)?)
```

### Status Code Guidance
- `bad_request` (400): invalid/missing input, download/read failures.
- `text_error(status, msg)`: propagate upstream status (e.g., download HTTP status).
- `HttpFailResult::as_fatal_error(...)`: unexpected/internal (compile, encode, deploy, signing, UTF-8 decode).

### Building Custom Errors
```rust
let out = HttpOutput::from_builder()
    .set_status_code(400)
    .set_content_type(WebContentType::Text)
    .set_content(msg.into_bytes())
    .build();
HttpFailResult::new(out, false, false)
```

### Checklist
- Export `errors` in `http_server/mod.rs`.
- Centralize conversions in `errors.rs`.
- Controllers import helpers; avoid inline `map_err`.
- Business logic stays in `scripts/`; controllers orchestrate only.
- When using `model:` in `http_route`, response structs derive `Serialize`, `Deserialize`, `MyHttpObjectStructure`.
