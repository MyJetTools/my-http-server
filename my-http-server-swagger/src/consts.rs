use proc_macro2::TokenStream;
use quote::quote;

pub fn get_http_field_type() -> TokenStream {
    quote!(data_types::HttpField)
}

pub fn get_http_data_type() -> TokenStream {
    quote!(data_types::HttpDataType)
}

pub fn get_http_array_element() -> TokenStream {
    quote!(data_types::ArrayElement)
}

pub fn get_use_documentation() -> TokenStream {
    quote!(
        use my_http_server::controllers::documentation::*;
    )
}

pub fn get_http_input_parameter() -> TokenStream {
    quote!(in_parameters::HttpInputParameter)
}

pub fn get_http_input_parameter_with_ns() -> TokenStream {
    quote!(my_http_server::controllers::documentation::in_parameters::HttpInputParameter)
}

pub fn get_http_parameter_input_src() -> TokenStream {
    quote!(in_parameters::HttpParameterInputSource)
}

pub fn get_http_context() -> TokenStream {
    quote!(my_http_server::HttpContext)
}

pub fn get_http_fail_result() -> TokenStream {
    quote!(my_http_server::HttpFailResult)
}

pub fn get_http_ok_result() -> TokenStream {
    quote!(my_http_server::HttpOkResult)
}

pub fn get_http_simple_type() -> TokenStream {
    quote!(data_types::HttpSimpleType)
}

pub fn get_http_action_description() -> TokenStream {
    quote!(HttpActionDescription)
}

pub fn get_http_action_description_with_ns() -> TokenStream {
    quote!(my_http_server::controllers::documentation::HttpActionDescription)
}

pub fn get_http_route() -> TokenStream {
    quote!(my_http_server::controllers::HttpRoute)
}

pub fn get_http_result() -> TokenStream {
    quote!(out_results::HttpResult)
}
