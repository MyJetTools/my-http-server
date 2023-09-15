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

    let data_structure_provider = match crate::http_object_structure::generate_data_provider(
        struct_name,
        generic_data.as_ref(),
    ) {
        Ok(result) => result,
        Err(err) => return (err.into_compile_error().into(), debug),
    };

    let get_http_data_structure =
        match crate::http_object_structure::generate_get_http_data_structure(
            struct_name,
            generic_data.as_ref(),
            &fields,
        ) {
            Ok(result) => result,
            Err(err) => return (err.into_compile_error().into(), debug),
        };

    let result = if let Some(generic) = GenericData::new(ast) {
        let generic_token_stream = generic.generic;
        let generic_ident = generic.generic_ident;
        quote! {
            #data_structure_provider

            impl<'s, #generic_token_stream> TryFrom<my_http_server::InputParamValue<'s>> for #struct_name #generic_ident {
                type Error = my_http_server::HttpFailResult;

                fn try_from(value: my_http_server::InputParamValue) -> Result<Self, Self::Error> {
                    value.from_json()
                }
            }

            impl #generic_token_stream TryFrom<my_http_server::HttpRequestBody> for #struct_name #generic_ident {
                type Error = my_http_server::HttpFailResult;

                fn try_from(value: my_http_server::HttpRequestBody) -> Result<Self, Self::Error> {
                    value.get_body_as_json()
                }
            }

            impl #generic_token_stream #struct_name #generic_ident {
                #get_http_data_structure
            }


        }
    } else {
        quote! {

            #data_structure_provider

            impl<'s> TryFrom<my_http_server::InputParamValue<'s>> for #struct_name {
                type Error = my_http_server::HttpFailResult;

                fn try_from(value: my_http_server::InputParamValue) -> Result<Self, Self::Error> {
                    value.from_json()
                }
            }

            impl TryFrom<my_http_server::HttpRequestBody> for #struct_name {
                type Error = my_http_server::HttpFailResult;

                fn try_from(value: my_http_server::HttpRequestBody) -> Result<Self, Self::Error> {
                    value.get_body_as_json()
                }
            }

            impl #struct_name {
                #get_http_data_structure
            }


        }
    };

    (result.into(), debug)
}
