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

**Not Modified (304) with an ETag:**
```rust
// The exact bytes get a strong ETag; a compressed variant of the same content, weak_etag(&hash)
let etag = strong_etag(&hash);

if ctx.request.if_none_match_matches(&etag) {
    return HttpOutput::as_not_modified()
        .add_header("ETag", etag)
        .add_header("Cache-Control", "no-cache")
        .into_ok_result(false);
}

HttpOutput::from_builder()
    .add_header("ETag", etag)
    .add_header("Cache-Control", "no-cache")
    .set_content_type(WebContentType::Html)
    .set_content(content)
    .into_ok_result(false)
```

- `strong_etag(hash)` → `"hash"`, `weak_etag(hash)` → `W/"hash"`. A strong ETag promises the same bytes: every content-coding of a file (gzip, br, …) needs one of its own, or they all share a weak one.
- `ctx.request.if_none_match_matches(&etag)` evaluates `If-None-Match` per RFC 9110 §13.1.2 with the weak comparison: `*` matches anything; otherwise the field is a list of tags (a comma inside quotes is part of the tag); `W/` is ignored on both sides; a header sent as several lines counts as one list. A tag without quotes is taken as the opaque tag: that is how this library wrote ETags earlier (base64 of a SHA-256, padding stripped, no quotes). A malformed field matches nothing. `if_none_match_matches(values, &etag)` does the same for header values from anywhere else.
- `as_not_modified()` has no body and no `Content-Type`. Add what the `200` would carry from `ETag`, `Cache-Control`, `Expires`, `Vary`, `Content-Location` (RFC 9110 §15.4.5) with `add_header`.

**Content types:** `WebContentType::Html`, `Css`, `JavaScript` and `Text` go with `; charset=utf-8`; `Json` is plain `application/json` (that type has no charset parameter). Any other type: `WebContentType::Raw(..)`. `WebContentType::detect_by_extension(path)` picks the type by the file extension (any letter case; `.mjs` is JavaScript, fonts, images, audio and video included) and returns `None` for an unknown one.

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

**Protobuf Response:**
```rust
HttpOutput::from_builder()
    .set_content(message.encode_to_vec())          // e.g. a prost message
    .set_content_type(WebContentType::Protobuf)    // application/x-protobuf
    .into_ok_result(true)
```

A protobuf **request** body needs nothing special: `application/x-protobuf` is not a content type the server parses, so a `#[http_body_raw]` field gets the bytes exactly as sent.

**Streaming Response:**
```rust
let (output, producer) = HttpOutput::as_stream(100);
// Hand `producer` to a task which sends the chunks: `producer.send(bytes).await`
output.get_result()
```

The response goes out chunked on HTTP/1.1 (as DATA frames on HTTP/2) and ends when the `producer` is dropped. To compress it, see [Streaming Response Compression](#streaming-response-compression-gzip) below.

**Response Compression (gzip):**

`HttpResultBuilder::with_compression(threshold, level)` opts the response into on-the-fly `gzip` compression. The body is compressed only when **both** conditions hold:

1. The current body size is **strictly greater** than `threshold` bytes.
2. The compressed payload is **≤ 80%** of the original size (otherwise compression isn't worth it and the original body is kept).

When the body is replaced with the compressed payload, the `Content-Encoding: gzip` header is added automatically. Otherwise the builder is returned untouched (no header, no allocation kept).

```rust
HttpOutput::as_json(large_model)
    .with_compression(4 * 1024, CompressionLevel::Balanced) // skip compression for bodies ≤ 4 KiB
    .into_ok_result(true)
    .into()
```

`level` trades CPU per response against bytes on the wire:

| `CompressionLevel` | gzip level | When |
|---|---|---|
| `Fast` | 1 | large or hot responses: the least CPU |
| `Balanced` (the `Default`) | 6 | a dynamic response compressed on every request |
| `Best` | 9 | responses produced rarely and fetched often: the smallest output |
| `Level(n)` | `n`, capped at 9 | an exact level. `Level(0)` only frames the bytes, so it never passes the 80% check |

Behaviour by `HttpOutput` variant:
- `Content`, `File` → body is compressed in place when the gates above pass.
- `Empty`, `Redirect` → no body to compress; method is a no-op.
- `Raw` → panics (the framework can't introspect a pre-built `MyHttpResponse`). A streamed response is compressed with its own method, below.

Pick `threshold` based on the response type — JSON / HTML payloads under a few KiB usually aren't worth compressing once you account for client-side decode cost. Setting `threshold = 0` forces compression to be attempted on every non-empty body (still gated by the 80% efficiency check).

**Encrypted Response (aes128gcm):**

`HttpResultBuilder::encode(&key)` encrypts the body: `Content-Encoding: aes128gcm` (RFC 8188), written by my-http-utils' `HttpBodyEncryptionKey::encrypt`, the encoding my-reverse-proxy reads its services' data in. `key` is an `HttpBodyEncryptionKey`, and a `BodyEncryptionKey` of the server's list derefs to one, so either goes in as it is. The action decides whether to encrypt and with which key. It usually takes the key by id from the same `Arc<BodyEncryptionKeys>` the server decrypts request bodies with (see [registration-and-startup](registration-and-startup.md)):

```rust
let key = action.app.body_encryption_keys.get_by_id("proxy").ok_or_else(|| {
    HttpFailResult::from((503u16, "No key to encrypt the answer with yet".to_string()))
})?;

HttpOutput::as_yaml(model)
    .encode(&key)
    .into_ok_result(false)
```

To answer with the key the client encrypted the request with, take it from the request:

```rust
let Some(info) = ctx.request.get_encryption_info() else {
    return Err(HttpFailResult::from((415u16, "The body has to come encrypted".to_string())));
};

HttpOutput::as_json(result)
    .encode(&info.key)
    .into_ok_result(false)
```

- **What it sends.** A fresh random salt every time, records of 4096 bytes and no key id in the header. What is encrypted is exactly the body the action wrote: unlike a request, an answer carries no clock in front of it, so the client opens it with `key.decrypt(&body)`, not as an `EncryptedBody`. `Content-Type` stays the type of the data before encryption. `Cache-Control: no-store` is added unless the action set a `Cache-Control` of its own.
- **Order.** The body is encrypted as it is at the call, so call `encode` after the content is set. After `with_compression` has gzipped the body, the header becomes `Content-Encoding: gzip, aes128gcm`, the codings in the order applied. Compressing after `encode` gains nothing, because encrypted bytes don't compress, and `with_compression` then leaves the body alone.
- **Variants.** `Content` and `File` with a body are encrypted, whatever the status code. An empty body, `Empty` and `Redirect` have nothing to encrypt, so `encode` does nothing. `Raw` panics.
- **Keys.** A `BodyEncryptionKey` is my-http-utils' `HttpBodyEncryptionKey` (32 random bytes) plus an id: `BodyEncryptionKey::from_base64("proxy", "<openssl rand -base64 32>")?`, `from_bytes("proxy", &bytes)?` or `BodyEncryptionKey::new("proxy", http_body_encryption_key)`. A bad key is an `HttpBodyEncryptionKeyError`, whose text never repeats what it was given. `get_by_id` returns a cheap clone: the key is shared, never printed by `Debug`, and wiped once the last clone is dropped.
- A streamed response (`HttpOutput::as_stream`) is not encrypted by `encode`.

**Streaming Response Compression (gzip):**

`HttpOutputAsStream::with_compression(&ctx.request, level)` gzips a streamed response on the fly, at the same `CompressionLevel` as above. The producer doesn't change — it keeps sending plain bytes, and the compression happens under the hood:

```rust
let (output, mut producer) = HttpOutput::as_stream(100);

tokio::spawn(async move {
    for chunk in chunks {
        producer.send(chunk).await?;   // plain bytes
    }
    // dropping `producer` ends the stream — the gzip footer is written then
});

output
    .with_header("Content-Type", "text/plain")
    .with_compression(&ctx.request, CompressionLevel::Fast)
    .get_result()
```

- **Negotiated.** The stream is compressed only when the client's `Accept-Encoding` takes `gzip` (or `x-gzip`, or `*`) with a non-zero weight; `gzip;q=0` refuses it. Otherwise it goes out as produced. Either way the response gets `Vary: Accept-Encoding`, unless the action already set a `Vary` that covers it.
- **Every chunk is flushed.** Each chunk the producer sends is compressed and sync-flushed, so it leaves immediately, and the client can decode everything it has received so far. A slow producer is still a stream.
- **The footer can't get lost.** The gzip footer (CRC32 + length) is written when the channel ends, by the stream that reads it, not by the producer. A producer that is simply dropped yields a complete gzip body.
- **No threshold, no efficiency check.** The headers leave before the first chunk exists, so there is no way to measure the gain, unlike `HttpResultBuilder::with_compression`. Each flush costs a few bytes, so a stream of tiny chunks (a heartbeat of a few bytes) can come out *larger* than it went in. Only opt in for streams whose chunks carry real payload.
- **Chunk size.** Chunks are compressed on the connection's task, so keep them reasonably sized: up to tens of KiB.
- **Headers.** A `Content-Length` set with `with_header` is dropped once the stream is compressed, since it counts the bytes produced, not the ones on the wire. A stream that already carries its own `Content-Encoding` is left exactly as it is, and is not compressed a second time.
- **Errors.** A stream that ends in an error is cut off without a footer. The client sees a broken body, not a complete-looking one.
