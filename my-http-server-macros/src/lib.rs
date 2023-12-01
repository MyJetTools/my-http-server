extern crate proc_macro;
use proc_macro::TokenStream;

use rust_extensions::date_time::DateTimeAsMicroseconds;
use syn;

mod action_builder;
mod as_token_stream;
mod consts;
mod enum_doc;
mod generic_utils;
mod http_input_field;
mod http_input_object_structure;
mod http_object_structure;
mod input_models;
mod property_type_ext;
mod types;

#[proc_macro_derive(
    MyHttpInput,
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
pub fn my_http_input_doc_derive(input: TokenStream) -> TokenStream {
    let ast = syn::parse(input).unwrap();
    let (result, debug) = crate::input_models::generate(&ast);

    if debug {
        println!("{}", result);
    }

    result
}

#[proc_macro_derive(MyHttpInputObjectStructure)]
pub fn my_http_input_object_derive(input: TokenStream) -> TokenStream {
    let ast = syn::parse(input).unwrap();
    let (result, debug) = crate::http_input_object_structure::generate(&ast);

    if debug {
        println!("{}", result);
    }

    result
}

#[proc_macro_derive(MyHttpObjectStructure, attributes(debug))]
pub fn my_http_output_object_derive(input: TokenStream) -> TokenStream {
    let ast = syn::parse(input).unwrap();
    let (result, debug) = crate::http_object_structure::generate(&ast);

    if debug {
        println!("{}", result);
    }
    result
}

#[proc_macro_derive(MyHttpStringEnum, attributes(http_enum_case))]
pub fn my_http_string_enum_derive(input: TokenStream) -> TokenStream {
    let ast = syn::parse(input).unwrap();
    match crate::enum_doc::generate(&ast, false) {
        Ok(result) => result,
        Err(err) => err.to_compile_error().into(),
    }
}

#[proc_macro_derive(MyHttpIntegerEnum, attributes(http_enum_case))]
pub fn my_http_integer_enum_derive(input: TokenStream) -> TokenStream {
    let ast = syn::parse(input).unwrap();
    match crate::enum_doc::generate(&ast, true) {
        Ok(result) => result,
        Err(err) => err.to_compile_error().into(),
    }
}

#[proc_macro_attribute]
pub fn http_route(attr: TokenStream, item: TokenStream) -> TokenStream {
    match crate::action_builder::build_action(attr, item) {
        Ok(result) => result,
        Err(err) => err.to_compile_error().into(),
    }
}

#[proc_macro]
pub fn pkg_compile_date_time(_input: TokenStream) -> TokenStream {
    let date = DateTimeAsMicroseconds::now().to_rfc3339();
    TokenStream::from(quote::quote!(#date))
}

#[proc_macro_attribute]
pub fn http_input_field(input: TokenStream, item: TokenStream) -> TokenStream {
    match crate::http_input_field::generate(input, item) {
        Ok(result) => result.into(),
        Err(err) => err.to_compile_error().into(),
    }
}
