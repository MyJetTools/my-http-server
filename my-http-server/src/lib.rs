#[cfg(feature = "macros")]
pub extern crate my_http_server_swagger as macros;

#[cfg(any(feature = "macros", feature = "controllers"))]
pub extern crate my_http_server_controllers as controllers;

#[cfg(any(feature = "web-sockets", feature = "signalr"))]
pub extern crate my_http_server_web_sockets as web_sockets;

#[cfg(feature = "signalr")]
pub extern crate my_http_server_signalr_middleware as signalr;

pub use my_http_server_core::*;
