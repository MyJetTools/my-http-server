// The attribute structs describe the full `#[http_query(...)]`-family markup so the server derive
// can parse it. Some fields (e.g. `description`) are part of that markup but are consumed by the
// schema/client derives in url-utils, not by the server-side parse — hence they are read there,
// not here.
#![allow(dead_code)]

mod http_form_data;
pub use http_form_data::*;
mod http_query;
pub use http_query::*;
mod http_path;
pub use http_path::*;
mod http_header;
pub use http_header::*;
mod http_body;
pub use http_body::*;
mod http_body_raw;
pub use http_body_raw::*;
mod ignore;
pub use ignore::*;
