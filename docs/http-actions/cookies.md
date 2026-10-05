# Working with Cookies

Cookies are set on the response builder: `set_cookie` adds one, `set_cookies` adds several. There is no need to build a `CookieJar` by hand — the builder keeps one internally.

```rust
use my_http_server::cookies::Cookie;

async fn handle_request(
    _action: &ActionName,
    input_data: InputModelName,
    _ctx: &HttpContext,
) -> Result<HttpOkResult, HttpFailResult> {
    HttpOutput::from_builder()
        // Cookie with options
        .set_cookie(
            Cookie::new("SessionId", "abc123")
                .set_path("/")
                .set_max_age(24 * 60 * 60),  // 24 hours
        )
        // A (name, value) tuple converts into a Cookie
        .set_cookie(("Test2".to_string(), "Value".to_string()))
        .set_cookie(("Test3", "Value".to_string()))
        .set_content_type(WebContentType::Json)
        .set_content(json_bytes)
        .build()
        .into_ok_result(true).into()
}
```

The same methods are available on every builder `HttpOutput` hands out (`as_json`, `as_text`, `as_html`, `as_redirect`, ...), so cookies can be attached to a ready-made response:

```rust
HttpOutput::as_json(response)
    .set_cookies([
        Cookie::new("SessionId", "abc123"),
        Cookie::new("Theme", "dark"),
    ])
    .into_ok_result(true)
    .into()
```
