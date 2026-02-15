use quote::quote;
use types_reader::StructProperty;

use crate::generic_utils::GenericData;

pub fn generate(ast: &syn::DeriveInput) -> (proc_macro::TokenStream, bool) {
    let struct_name = &ast.ident;

    let mut debug = false;

    let fields = match StructProperty::read(ast) {
        Ok(result) => result,
        Err(err) => return (err.into_compile_error().into(), debug),
    };

    for field in &fields {
        if field.attrs.has_attr("debug") {
            debug = true;
        }
    }

    let generic_data = GenericData::new(ast);

    let get_http_data_structure =
        match crate::http_object_structure::generate_get_http_data_structure(
            struct_name,
            generic_data.as_ref(),
            &fields,
        ) {
            Ok(result) => result,
            Err(err) => return (err.into_compile_error().into(), debug),
        };

    let data_structure_provider = match crate::http_object_structure::generate_data_provider(
        struct_name,
        generic_data.as_ref(),
        get_http_data_structure,
    ) {
        Ok(result) => result,
        Err(err) => return (err.into_compile_error().into(), debug),
    };

    let from_encoded_param_value_content = generate_from_encoded_param_value_content();

    let from_http_request_body = generate_from_http_request_body();

    let from_form_data = generate_from_form_data(&struct_name);

    let result = if let Some(generic) = GenericData::new(ast) {
        let generic_token_stream = generic.generic;
        let generic_ident = generic.generic_ident;

        quote! {
            #data_structure_provider

            impl<'s, #generic_token_stream> TryFrom<my_http_server::EncodedParamValue<'s>> for #struct_name #generic_ident {
                #from_encoded_param_value_content
            }

            impl #generic_token_stream TryFrom<my_http_server::HttpRequestBody> for #struct_name #generic_ident {
                #from_http_request_body
            }

            impl #generic_ident TryInto<#struct_name> for &'s my_http_server::FormDataItem #generic_ident {
                #from_form_data
            }


        }
    } else {
        quote! {
            #data_structure_provider

            impl<'s> TryFrom<my_http_server::EncodedParamValue<'s>> for #struct_name {
                #from_encoded_param_value_content
            }

            impl TryFrom<my_http_server::HttpRequestBody> for #struct_name {
                #from_http_request_body
            }

            impl<'s> TryInto<#struct_name> for &'s my_http_server::FormDataItem<'s>  {
                #from_form_data
            }

        }
    };

    (result.into(), debug)
}

fn generate_from_encoded_param_value_content() -> proc_macro2::TokenStream {
    quote::quote! {
        type Error = my_http_server::HttpFailResult;

        fn try_from(value: my_http_server::EncodedParamValue) -> Result<Self, Self::Error> {
            value.from_json()
        }
    }
}

fn generate_from_http_request_body() -> proc_macro2::TokenStream {
    quote::quote! {
        type Error = my_http_server::HttpFailResult;

        fn try_from(value: my_http_server::HttpRequestBody) -> Result<Self, Self::Error> {
            value.get_body_as_json()
        }
    }
}

fn generate_from_form_data(struct_name: &syn::Ident) -> proc_macro2::TokenStream {
    quote::quote! {
        type Error = my_http_server::HttpFailResult;

        fn try_into(self) -> Result<#struct_name, Self::Error> {
            use my_http_server::data_src::*;
            match self {
                my_http_server::FormDataItem::ValueAsString { value, name } => Ok(
                    my_http_server::convert_from_str::to_json_from_str(name, value, SRC_FORM_DATA)?,
                ),
                my_http_server::FormDataItem::File {
                    name,
                    file_name: _,
                    content_type: _,
                    content,
                } => Ok(my_http_server::convert_from_str::to_json(
                    name,
                    &Some(content),
                    SRC_FORM_DATA,
                )?),
            }
        }
    }
}
