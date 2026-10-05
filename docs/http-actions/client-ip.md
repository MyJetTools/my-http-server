# Extracting Client IP

**Always use `get_ip()` — never parse `X-Forwarded-For` / `X-Real-Ip` manually.**

For HTTP actions (`HttpContext`) and for WebSocket `connected` callback (`MyWebSocketHttpRequest`) there is a ready method that handles proxy headers and falls back to the socket peer address:

```rust
// HTTP action
async fn handle_request(
    _action: &ActionName,
    input_data: InputModelName,
    ctx: &HttpContext,
) -> Result<HttpOkResult, HttpFailResult> {
    let ip: String = ctx.request.get_ip().get_real_ip_as_string();
    // or, if a &str is enough:
    // let ip: &str = ctx.request.get_ip().get_real_ip();
    // ...
}

// WebSocket connected callback
async fn connected(
    &self,
    my_web_socket: Arc<MyWebSocket>,
    http_request: MyWebSocketHttpRequest,
    _disconnect_timeout: Duration,
) -> Result<(), WebSocketConnectedFail> {
    let ip = http_request.get_ip().get_real_ip_as_string();
    // ...
}
```

`RequestIp` (returned by `get_ip()`) already:
- parses `X-Forwarded-For` (returns the first/left-most IP — the original client, before proxies)
- falls back to the socket peer address when no proxy headers are set

Do **not** write the following kind of code — it duplicates what the library already does and will not pick up future header support added in the library:

```rust
// ❌ Don't do this
let ip = http_request
    .get_headers()
    .get("X-Forwarded-For")
    .or_else(|| http_request.get_headers().get("X-Real-Ip"))
    .and_then(|v| v.to_str().ok())
    .map(|v| v.split(',').next().unwrap_or(v).trim().to_string())
    .unwrap_or_default();
```

**Note:** extracting `CF-IPCountry` (Cloudflare country) or other non-IP headers is unrelated — read those from `get_headers()` directly.
