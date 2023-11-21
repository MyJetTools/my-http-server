use std::str::FromStr;

use proc_macro::TokenStream;
use types_reader::EnumCase;

use crate::enum_doc::enum_json::{EnumJson, HTTP_ENUM_ATTR_NAME};

use super::generate_default::generate_default_as_str_fn;

pub fn generate(ast: &syn::DeriveInput, as_integer: bool) -> Result<TokenStream, syn::Error> {
    let struct_name = &ast.ident;
    let struct_name_as_str = struct_name.to_string();

    let src_fields = EnumCase::read(ast)?;

    let mut fields = Vec::new();

    let mut default_str_value = None;
    let mut default_case_value = None;

    for src_field in src_fields {
        let name = src_field.get_name_ident().to_string();
        if let Some(enum_json) = EnumJson::new(src_field) {
            if enum_json.is_default_value {
                default_str_value = Some(enum_json.get_enum_case_str_value()?);
                default_case_value = Some(enum_json.get_enum_case_value());
            }

            fields.push(enum_json);
        } else {
            panic!(
                "Enum case {} does not have #[{}] attribute",
                name, HTTP_ENUM_ATTR_NAME
            )
        }
    }

    //Default Trait

    let default_trait = if let Some(default_case) = &default_case_value {
        let default_case = proc_macro2::TokenStream::from_str(default_case).unwrap();

        quote::quote! {
            impl std::default::Default for #struct_name{
                fn default() -> Self {
                    Self::#default_case
                }
            }
        }
    } else {
        quote::quote!()
    };

    let http_fail_result = crate::consts::get_http_fail_result();

    let create_default_impl = if default_case_value.is_some() {
        quote::quote!(Ok(Self::default()))
    } else {
        let err = format!(
            "Type {} does not have default value to create",
            struct_name_as_str
        );
        quote::quote! {
            Err(#http_fail_result::as_forbidden(Some(#err.to_string())))
        }
    };

    let impl_from_str =
        match super::impl_from_str_trait::generate(&struct_name_as_str.as_str(), fields.as_slice())
        {
            Ok(impl_from_str) => impl_from_str,
            Err(err) => vec![err.to_compile_error()],
        };

    let impl_from_i32 = match super::impl_from_i32::generate(fields.as_slice()) {
        Ok(impl_from_i32) => impl_from_i32,
        Err(err) => vec![err.to_compile_error()],
    };

    let use_documentation = crate::consts::get_use_documentation();

    let enum_cases = generate_enum_cases(&fields)?;

    let default_as_str_fn = generate_default_as_str_fn(default_str_value.as_ref());

    let enum_type = if as_integer {
        quote::quote!(EnumType::Integer)
    } else {
        quote::quote!(EnumType::String)
    };

    let result = quote::quote! {
        impl #struct_name{

            pub fn create_default() -> Result<Self,#http_fail_result>{
                #create_default_impl
            }

            #default_as_str_fn
        }

        #default_trait

        impl std::str::FromStr for #struct_name{
            type Err = #http_fail_result;
            fn from_str(src:&str)->Result<Self,Self::Err>{
                #(#impl_from_str)*
            }
        }

        impl From<i32> for #struct_name{
            fn from(src: i32) -> Self {
                match src {
                #(#impl_from_i32)*
                }
            }
        }

        impl<'s> TryInto<#struct_name> for my_http_server::EncodedParamValue<'s>{
            type Error = my_http_server::HttpFailResult;

            fn try_into(self) -> Result<#struct_name, Self::Error> {
                self.from_str()
            }

        }


        impl my_http_server::controllers::documentation::DataTypeProvider for #struct_name {
            fn get_data_type() -> my_http_server::controllers::documentation::data_types::HttpDataType {
                #use_documentation;

                let mut __es = data_types::HttpEnumStructure{
                    struct_id: #struct_name_as_str,
                    enum_type: #enum_type,
                    cases: vec![],
                };

                #(#enum_cases)*

                __es.into_http_data_type_object()
            }

            fn get_generic_type() -> Option<&'static str> {
                None
             }
        }



    };

    Ok(result.into())
}

fn generate_enum_cases(cases: &[EnumJson]) -> Result<Vec<proc_macro2::TokenStream>, syn::Error> {
    let mut result = Vec::with_capacity(cases.len());
    for case in cases {
        let id = proc_macro2::Literal::isize_unsuffixed(case.get_id()?);
        let value = case.get_enum_case_value();
        let description = case.description()?;

        result.push(quote::quote! {
            __es.cases.push(data_types::HttpEnumCase{
                id: #id,
                value: #value,
                description: #description
            });
        });
    }

    Ok(result)
}
