# Shared models when a UI consumes this API

**Rule:** if this REST API is consumed by a UI (a Dioxus client-side app, or any wasm client), do **not** keep the input and response models locally inside the service. Move them into a dedicated `rest-api-shared` crate and put the models there. One source of truth — both sides use it **verbatim**, so the wire contract can never drift between server and client.

If the API is **not** consumed by any UI (internal service-to-service only), the models may stay local in the service.

**Setting up the crate:**

`rest-api-shared` is a separate crate sitting next to the service, wired as a `path` dependency, and kept **wasm-clean**: it depends **only** on `my-http-utils` (plus `serde` / `serde_json`) and **never** on `my-http-server`.

```toml
# rest-api-shared/Cargo.toml
[dependencies]
my-http-utils = { ... }              # schema + MyHttpInput parsing; the ONLY MyJetTools dep
serde = { version = "1", features = ["derive"] }
serde_json = "1"

[features]
server = ["my-http-utils/server"]    # forwards server-side request parsing
```

**Models:**

- Request models  → `#[derive(MyHttpInput)]`
- Response models → `#[derive(Serialize, Deserialize, MyHttpObjectStructure, Clone, Debug, PartialEq)]`

```rust
// rest-api-shared/src/...
#[derive(MyHttpInput)]
pub struct LoginRequest {
    #[http_body(name = "email", description = "User email")]
    pub email: String,
    #[http_body(name = "password", description = "User password")]
    pub password: String,
}

#[derive(Serialize, Deserialize, MyHttpObjectStructure, Clone, Debug, PartialEq)]
pub struct LoginResponse {
    pub token: String,
    pub expires_at: i64,
}
```

Keep these structs as **pure data** — no methods, no `impl` blocks. Any derived or presentational logic belongs in **extension traits on the consumer side** (on the server, or in the UI), not on the shared struct.

> ⚠️ Do **not** put `///` doc-comments on the fields of these structs — the derive proc-macro panics on them. Document `#[http_*]` fields with the `description = "..."` attribute argument (as above) instead.

**Wiring each side:**

- **REST-API service** — enable the `server` feature so it gets server-side request parsing, then import the models from the shared crate in your actions:

  ```toml
  # service Cargo.toml
  rest-api-shared = { path = "../rest-api-shared", features = ["server"] }
  ```

- **UI (wasm client)** — depend on the same crate **without** the `server` feature; the client only needs the schema and the `FlUrl` request builder:

  ```toml
  # UI Cargo.toml
  rest-api-shared = { path = "../rest-api-shared" }   # no "server" feature
  ```

Because both sides pull the identical `MyHttpInput` / `MyHttpObjectStructure` definitions, the request the UI builds and the request the server parses are guaranteed to match.
