#[cfg(feature = "macros")]
pub extern crate my_http_server_swagger as macros;

#[cfg(any(feature = "macros", feature = "controllers"))]
pub extern crate my_http_server_controllers as controllers;

#[cfg(any(feature = "websocket"))]
pub extern crate my_http_server_web_sockets as web_sockets;

#[cfg(feature = "signal-r")]
pub mod signal_r{
    extern crate my_http_server_signal_r_middleware;
    pub use my_http_server_signal_r_middleware::*;
    pub extern crate signal_r_macros as macros;
}


#[cfg(feature = "static-files")]
pub use static_files_middleware::*;

pub use my_http_server_core::*;
