use std::str::FromStr;

use proc_macro2::TokenStream;

use super::attributes::HttpResult;

pub fn generate(results: &Option<Vec<HttpResult>>) -> TokenStream {
    let mut fields = Vec::new();

    if let Some(http_results) = results {
        let http_result_type = crate::consts::get_http_result();

        for http_result in http_results {
            let description = &http_result.description;

            let http_code = proc_macro2::Literal::u16_unsuffixed(http_result.status_code);

            let data_type = compile_data_type(http_result);

            fields.push(quote::quote! {
                #http_result_type{
                    nullable: false,
                    description: #description.to_string(),
                    http_code: #http_code,
                    data_type: #data_type,
                }
            });
        }
    }

    quote::quote!(vec![#(#fields),*]).into()
}

fn generate_as_object(object_name: &str, into_structure: TokenStream) -> TokenStream {
    if let Some(index) = object_name.find('<') {
        let mut result_obj_name = String::new();

        result_obj_name.push_str(&object_name[..index]);
        result_obj_name.push_str("::");
        result_obj_name.push_str(&object_name[index..]);

        let object_name = TokenStream::from_str(result_obj_name.as_str()).unwrap();
        quote::quote!(#object_name::get_http_data_structure().#into_structure())
    } else {
        let object_name = TokenStream::from_str(object_name).unwrap();
        quote::quote!(#object_name::get_http_data_structure().#into_structure())
    }
}

fn generate_as_array(object_name: &str, into_structure: TokenStream) -> TokenStream {
    if let Some(index) = object_name.find('<') {
        let mut result_obj_name = String::new();

        result_obj_name.push_str(&object_name[..index]);
        result_obj_name.push_str("::");
        result_obj_name.push_str(&object_name[index..]);

        let object_name = TokenStream::from_str(result_obj_name.as_str()).unwrap();
        quote::quote!(#object_name::get_http_data_structure().#into_structure())
    } else {
        let object_name = TokenStream::from_str(object_name).unwrap();
        quote::quote!(#object_name::get_http_data_structure().#into_structure())
    }
}

fn compile_data_type(http_result: &HttpResult) -> TokenStream {
    let http_data_type = crate::consts::get_http_data_type();

    if let Some(result_type) = &http_result.result_type {
        match result_type {
            super::attributes::HttpResultModel::Object(object_name) => {
                generate_as_object(object_name, quote::quote!(into_http_data_type_object))
            }
            super::attributes::HttpResultModel::Array(object_name) => {
                generate_as_array(object_name, quote::quote!(into_http_data_type_array))
            }
            super::attributes::HttpResultModel::ArrayOfSimpleType(type_name) => {
                let http_array_element = crate::consts::get_http_array_element();
                let http_simple_type = crate::consts::get_http_simple_type();
                let type_name = TokenStream::from_str(type_name).unwrap();
                quote::quote!(#http_data_type::ArrayOf(#http_array_element::SimpleType(#http_simple_type::#type_name)))
            }
            super::attributes::HttpResultModel::SimpleType(type_name) => {
                let http_simple_type = crate::consts::get_http_simple_type();
                let type_name = TokenStream::from_str(type_name).unwrap();
                quote::quote!(#http_data_type::SimpleType(#http_simple_type::#type_name))
            }
        }
    } else {
        quote::quote!(#http_data_type::None)
    }
}
