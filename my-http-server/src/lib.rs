#[cfg(feature = "macros")]
pub extern crate my_http_server_swagger as macros;

#[cfg(any(feature = "macros", feature = "controllers"))]
pub extern crate my_http_server_controllers as controllers;

#[cfg(any(feature = "web-sockets", feature = "signal-r"))]
pub extern crate my_http_server_web_sockets as web_sockets;

#[cfg(feature = "signal-r")]
pub extern crate my_http_server_signal_r_middleware as signal_r;

#[cfg(feature = "static-files")]
pub use static_files_middleware::*;

pub use my_http_server_core::*;
