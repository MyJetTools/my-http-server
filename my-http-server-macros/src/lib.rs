extern crate proc_macro;
use proc_macro::TokenStream;

use syn;
use types_reader::rust_extensions::date_time::DateTimeAsMicroseconds;

mod attributes;
mod consts;
mod http_route;
mod input_models;

/// Server-only companion of `my_http_utils::macros::MyHttpInput`: generates `parse_http_input`
/// on the same field markup. The schema and the client request builder come from my-http-utils;
/// this adds the server-side parsing of an incoming request. Use both derives together on
/// server model crates.
#[proc_macro_derive(
    MyHttpInputServer,
    attributes(
        http_query,
        http_header,
        http_path,
        http_form_data,
        http_body,
        http_body_raw,
        debug,
    )
)]
pub fn my_http_input_server_derive(input: TokenStream) -> TokenStream {
    let ast = syn::parse(input).unwrap();
    let mut debug = false;
    let result = match crate::input_models::generate_server(&ast, &mut debug) {
        Ok(result) => result,
        Err(err) => err.to_compile_error().into(),
    };

    if debug {
        println!("{}", result);
    }

    result
}

#[proc_macro_attribute]
pub fn http_route(attr: TokenStream, item: TokenStream) -> TokenStream {
    match crate::http_route::build_action(attr, item) {
        Ok(result) => result,
        Err(err) => err.to_compile_error().into(),
    }
}

#[proc_macro]
pub fn pkg_compile_date_time(_input: TokenStream) -> TokenStream {
    let date = DateTimeAsMicroseconds::now().to_rfc3339();
    TokenStream::from(quote::quote!(#date))
}
