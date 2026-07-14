#[cfg(feature = "macros")]
pub mod macros {
    // Model description AND parsing come from my-http-utils (`MyHttpInput` emits the schema, the
    // client request builder, and — under the `server` feature — the sync `parse` / `READS_BODY`),
    // so the same models are shared with clients such as fl-url. Only server-glue macros stay
    // local: `http_route` (routing) and `pkg_compile_date_time`.
    pub use my_http_server_macros::{http_route, pkg_compile_date_time};
    pub use my_http_utils::macros::*;
}

#[cfg(any(feature = "macros", feature = "controllers"))]
pub extern crate my_http_server_controllers as controllers;

#[cfg(any(feature = "websocket"))]
pub extern crate my_http_server_web_sockets as web_sockets;

#[cfg(feature = "signal-r")]
pub mod signal_r {
    extern crate my_http_server_signal_r_middleware;
    pub use my_http_server_signal_r_middleware::*;
    pub extern crate signal_r_macros as macros;
}

#[cfg(feature = "static-files")]
pub use static_files_middleware::*;

pub use my_http_server_core::*;
