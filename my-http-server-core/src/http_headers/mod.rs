mod headers;
pub use headers::*;
mod header_value;
pub use header_value::*;
mod common_headers;
pub use common_headers::*;

pub trait AddHttpHeaders {
    fn add_header(&mut self, header_name: impl Into<String>, header_name: impl Into<String>);
}
