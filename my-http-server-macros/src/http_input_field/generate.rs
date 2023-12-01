use std::str::FromStr;

use proc_macro::TokenStream;
use types_reader::TokensObject;

pub fn generate(attr: TokenStream, input: TokenStream) -> Result<TokenStream, syn::Error> {
    let input: proc_macro2::TokenStream = input.into();
    let src = quote::quote!(#input);

    let tuple_struct = types_reader::SingleValueTupleStruct::new(input)?;

    let attr: proc_macro2::TokenStream = attr.into();

    let attr = TokensObject::new(attr.into(), &|| None)?;

    let open_api_type = attr.try_get_value_from_single_or_named("open_api_type");

    let open_api_type = match open_api_type {
        Some(swagger_type_obj) => {
            let swagger_type = swagger_type_obj.as_string()?.as_str();

            match swagger_type {
                "String" => proc_macro2::TokenStream::from_str("String").unwrap(),
                "Password" => proc_macro2::TokenStream::from_str("Password").unwrap(),
                _ => {
                    return Err(swagger_type_obj
                        .throw_error("Unknown swagger type. String and Password are supported"))
                }
            }
        }
        None => proc_macro2::TokenStream::from_str("String").unwrap(),
    };

    let struct_name = &tuple_struct.name_ident;

    let tp = &tuple_struct.type_ident;

    let result = quote::quote! {

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
