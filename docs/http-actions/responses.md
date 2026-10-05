# Response Types

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
    .set_status_code(201)
    .add_header("Location", "/api/resource/123")
    .set_content_type(WebContentType::Json)
    .set_cookie(cookie)
    .set_content(json_bytes)
    .build()
    .into_ok_result(true).into()
```

**Streaming Response:**
```rust
let (output, producer) = HttpOutput::as_stream(100);
// Hand `producer` to a task which sends the chunks: `producer.send(bytes).await`
output.get_result()
```

**Response Compression (gzip):**

`HttpResultBuilder::with_compression(threshold)` opts the response into on-the-fly `gzip` compression. The body is compressed only when **both** conditions hold:

1. The current body size is **strictly greater** than `threshold` bytes.
2. The compressed payload is **≤ 80%** of the original size (otherwise compression isn't worth it and the original body is kept).

When the body is replaced with the compressed payload, the `Content-Encoding: gzip` header is added automatically. Otherwise the builder is returned untouched (no header, no allocation kept).

```rust
HttpOutput::as_json(large_model)
    .with_compression(4 * 1024)            // skip compression for bodies ≤ 4 KiB
    .into_ok_result(true)
    .into()
```

Behaviour by `HttpOutput` variant:
- `Content`, `File` → body is compressed in place when the gates above pass.
- `Empty`, `Redirect` → no body to compress; method is a no-op.
- `Raw` → panics (the framework can't introspect a pre-built `MyHttpResponse`).

Pick `threshold` based on the response type — JSON / HTML payloads under a few KiB usually aren't worth compressing once you account for client-side decode cost. Setting `threshold = 0` forces compression to be attempted on every non-empty body (still gated by the 80% efficiency check).
