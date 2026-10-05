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
