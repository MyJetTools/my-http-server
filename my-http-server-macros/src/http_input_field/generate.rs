use std::str::FromStr;

use proc_macro::TokenStream;
use types_reader::macros::{MacrosEnum, MacrosParameters};
use types_reader::TokensObject;

#[derive(MacrosParameters)]
pub struct MacrosParameters {
    #[default]
    pub open_api_type: Option<OpenApiType>,
}

#[derive(MacrosEnum)]
pub enum OpenApiType {
    #[default]
    String,
    Password,
}

impl OpenApiType {
    pub fn get_macros_stream(&self) -> proc_macro2::TokenStream {
        match self {
            Self::String => proc_macro2::TokenStream::from_str("String").unwrap(),
            Self::Password => proc_macro2::TokenStream::from_str("Password").unwrap(),
        }
    }
}

pub fn generate(attr: TokenStream, input: TokenStream) -> Result<TokenStream, syn::Error> {
    let input: proc_macro2::TokenStream = input.into();
    let src = quote::quote!(#input);

    let tuple_struct = types_reader::SingleValueTupleStruct::new(input)?;

    let attr: proc_macro2::TokenStream = attr.into();

    let attr = TokensObject::new(attr.into())?;

    let attrs: MacrosParameters = (&attr).try_into()?;

    let open_api_type = match attrs.open_api_type {
        Some(open_api_type) => open_api_type.get_macros_stream(),
        None => OpenApiType::default().get_macros_stream(),
    };

    let struct_name = &tuple_struct.name_ident;

    let tp = &tuple_struct.type_ident;

    let result = quote::quote! {
        #[derive(Debug)]
        #src

        impl  my_http_server::controllers::documentation::DataTypeProvider for #struct_name {
            fn get_data_type() -> my_http_server::controllers::documentation::HttpDataType {
                my_http_server::controllers::documentation::HttpDataType::SimpleType(
                    my_http_server::controllers::documentation::HttpSimpleType::#open_api_type,
                )
            }
        }

            impl #struct_name {
                fn new(src: &str) -> Result<Self, my_http_server::HttpFailResult> {
                    let src = process_value(src)?;
                    Ok(Self(src.to_string()))

                }
                pub fn as_str(&self) -> &str {
                    self.0.as_str()
                }

                pub fn validate(src: &str)-> Result<rust_extensions::StrOrString, my_http_server::HttpFailResult>{
                    process_value(src)
                }
            }

            impl Into<#tp> for #struct_name {
                fn into(self) -> #tp {
                    self.0
                }
            }

            impl<'s> TryInto<#struct_name> for my_http_server::EncodedParamValue<'s> {
                type Error = my_http_server::HttpFailResult;

                fn try_into(self) -> Result<#struct_name, Self::Error> {
                    let src = self.as_str()?;
                    #struct_name::new(src.as_str())
                }
            }

        impl<'s> TryInto<#struct_name> for my_http_server::FormDataItem<'s> {
            type Error = my_http_server::HttpFailResult;

                fn try_into(self) -> Result<#struct_name, Self::Error> {
                    match self {
                        my_http_server::FormDataItem::ValueAsString { name, value } => {
                            match value {
                                Some(value) => return #struct_name::new(value),
                                None =>{
                                    return Err(my_http_server::HttpFailResult::required_parameter_is_missing(
                                        name, "FormData"
                                    ))
                                },
                            }


                        }
                        my_http_server::FormDataItem::File { name, .. } => Err(my_http_server::HttpFailResult::as_validation_error(
                            format!("{name} Can not be read from FormData file"),
                        )),
                    }
                }
        }

      impl<'s> TryInto<#struct_name> for my_http_server::HeaderValue<'s> {
          type Error = my_http_server::HttpFailResult;

          fn try_into(self) -> Result<#struct_name, Self::Error> {
            let src = self.as_str()?;
            #struct_name::new(src)
          }
      }

    };

    Ok(result.into())
}
