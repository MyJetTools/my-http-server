# Controller Registration and Server Startup

## Controller Registration

Actions are registered in `src/http_server/build_controllers.rs`, on a `ControllersMiddleware`. Every `register_*_action` takes the action in an `Arc`:

```rust
use std::sync::Arc;

use my_http_server::controllers::ControllersMiddleware;

use crate::app::AppContext;

pub fn build_controllers(app: &Arc<AppContext>) -> ControllersMiddleware {
    // (authorization, auth error factory) - None, None: no global authorization
    let mut result = ControllersMiddleware::new(None, None);

    // POST actions
    result.register_post_action(Arc::new(
        super::controllers::controller_group::PostAction::new(app.clone()),
    ));

    // GET actions
    result.register_get_action(Arc::new(
        super::controllers::controller_group::GetAction::new(app.clone()),
    ));

    // PUT actions
    result.register_put_action(Arc::new(
        super::controllers::controller_group::UpdateAction::new(app.clone()),
    ));

    // DELETE actions
    result.register_delete_action(Arc::new(
        super::controllers::controller_group::DeleteAction::new(app.clone()),
    ));

    result
}
```

Global authorization is the first argument of `ControllersMiddleware::new` — `Some(ControllersAuthorization::BearerAuthentication { .. })` (or `BasicAuthentication`, `ApiKeys`); the second is an optional `AuthErrorFactory` that shapes the response to a failed check.

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

## Module Organization

**Controller group module (`controllers/{group}/mod.rs`):**
```rust
pub mod action_name_action;
pub use action_name_action::*;
```

**Main controllers module (`controllers/mod.rs`):**
```rust
pub mod controller_group;
```

## Server Startup

The server is created and started in `src/http_server/start_up.rs`:

```rust
use std::{net::SocketAddr, sync::Arc};

use my_http_server::controllers::swagger::SwaggerMiddleware;
use my_http_server::MyHttpServer;

use crate::app::AppContext;

pub fn start(app: &Arc<AppContext>) {
    let mut http_server = MyHttpServer::new(SocketAddr::from(([0, 0, 0, 0], 8000)));

    let controllers = Arc::new(super::build_controllers(app));

    // Swagger UI at /swagger, generated from the registered actions
    let swagger_middleware =
        SwaggerMiddleware::new(controllers.clone(), crate::app::APP_NAME, crate::app::APP_VERSION);

    http_server.add_middleware(Arc::new(swagger_middleware));
    http_server.add_middleware(controllers);

    http_server.start(app.app_states.clone(), my_logger::LOGGER.clone());
}
```

- Middlewares answer in the order they are added: swagger first, then the controllers.
- `start` spawns the server and returns; calling it a second time panics.
- Settings go before `start`: `set_tcp_nodelay(bool)` (`true` by default — `TCP_NODELAY` on every accepted TCP connection, so Nagle's algorithm and the client's delayed ACK do not hold back the end of a response; `false` leaves Nagle on; Unix sockets are not affected), `set_header_read_timeout(duration)` (30 s by default — how long an HTTP/1 client has to send the head of a request, and how long an idle keep-alive connection waits for the next one; the connection is closed past it; the body, the handler, websockets and HTTP/2 are not covered; `None` switches it off), `set_body_read_timeout(duration)` (off by default), `set_max_decompressed_body_size(bytes)` (64 MiB by default), `set_body_encryption_keys(Arc<BodyEncryptionKeys>)` (none by default).
- Encrypted request bodies: give the server an `Arc<BodyEncryptionKeys>` and keep a clone. A body sent with `Content-Encoding: aes128gcm` (RFC 8188) is decrypted when it is read, like a compressed one: each key is tried in turn. The body is my-http-utils' `EncryptedBody`, the standard fl-url and the services behind my-reverse-proxy share: what is encrypted starts with the sender's clock (unix microseconds, an `i64` in 8 bytes, little-endian). The server takes the clock off, and the action gets the rest as the body. A client sends such a body with fl-url's `encrypted(&key)` or my-http-utils' `HttpRequestBody::encrypt`; with `compress()` first it goes as `Content-Encoding: gzip, aes128gcm`, and the server decrypts, then decompresses. An empty body is empty bytes to the standard, header or not: it reaches the action empty, with no encryption info. How the body came is on the request: `ctx.request.get_encryption_info()` gives the key that opened it, the sender's clock (`timestamp`), its salt (16 random bytes, fresh in every body, the thing a replay check remembers), its record size and the key id from its header. It is `None` for a plain body, so an action that takes encrypted bodies only refuses a plain one with it. The salt and the rest travel in the first bytes of the body, not in HTTP headers, so once the body is decrypted this is the only place they are kept. `keys.set(vec![...])` replaces the keys while the server runs, so keys that arrive after the start, like my-reverse-proxy's after `/api/initialize`, are used from the next request on. An encrypted body is never read as plain. With no key (never set, or the list is empty) it is answered `415` with `Accept-Encoding: gzip, deflate, br, zstd` (RFC 9110 §15.5.16). When no key opens it, when it is damaged or cut short, or when what opened is shorter than the 8 bytes of the clock, it is answered `400`. A replay is refused before the action runs, with `403`: a body whose clock is more than 10 seconds off the server's, either way (`BodyEncryptionKeys::new(keys).with_max_clock_skew(..)` sets the window), or whose salt was already accepted. Salts are kept with the keys, so every server given the same `Arc` shares them. Only the header makes a body encrypted: a body without it is taken as it is. A `#[http_body_as_stream]` body is handed on as it arrived. Responses are encrypted by the action with `encode`, see [responses](responses.md). The format on the wire and the replay checks are in [encrypted-requests](encrypted-requests.md).

```rust
// In AppContext::new - the actions take their keys by id from the same list:
let body_encryption_keys = Arc::new(BodyEncryptionKeys::new(vec![
    BodyEncryptionKey::from_base64("proxy", &settings.proxy_key).unwrap(),
]));

// In start:
http_server.set_body_encryption_keys(app.body_encryption_keys.clone());
```
- A failed `accept()` does not stop the server: an error such as `EMFILE` (out of file descriptors) is logged with `write_error` and accepting is tried again 100 ms later; a connection the peer dropped before it was accepted is logged at debug level.
