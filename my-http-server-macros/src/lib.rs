extern crate proc_macro;
use proc_macro::TokenStream;

mod compile_date_time;
mod consts;
mod http_route;

// Model description AND parsing now come entirely from `my_http_utils::macros::MyHttpInput`
// (schema + client builder always, and the server-side sync `parse` + `READS_BODY` under the
// `server` feature). This crate keeps only the server-glue macros: `#[http_route]` (routing /
// action wiring) and `pkg_compile_date_time`.

#[proc_macro_attribute]
pub fn http_route(attr: TokenStream, item: TokenStream) -> TokenStream {
    match crate::http_route::build_action(attr, item) {
        Ok(result) => result,
        Err(err) => err.to_compile_error().into(),
    }
}

#[proc_macro]
pub fn pkg_compile_date_time(_input: TokenStream) -> TokenStream {
    let date = crate::compile_date_time::now_as_rfc3339();
    TokenStream::from(quote::quote!(#date))
}
