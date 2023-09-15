use types_reader::StructProperty;

use super::{HttpInputSource, InputField};

const HTTP_PATH_NAME: &'static str = "http_path";

pub trait InputModelStructPropertyExt {
    fn try_into_input_field(&self) -> Result<Option<InputField>, syn::Error>;
    fn get_struct_field_name_as_token_stream(&self) -> proc_macro2::TokenStream;
}

impl<'s> InputModelStructPropertyExt for StructProperty<'s> {
    fn try_into_input_field(&self) -> Result<Option<InputField>, syn::Error> {
        let mut result = None;
        for attr_name in self.attrs.get_attr_names() {
            let attr = self.attrs.get_attr(attr_name)?;
            match attr_name.as_str() {
                "http_query" => {
                    if result.is_some() {
                        return Err(syn::Error::new(
                            self.get_field_name_ident().span(),
                            format!("Please specify single attribute as http_query, http_header, http_path, http_form_data or http_body."),
                        ));
                    }
                    result = Some(InputField::new(self, attr, HttpInputSource::Query));
                }
                "http_header" => {
                    if result.is_some() {
                        return Err(syn::Error::new(
                            self.get_field_name_ident().span(),
                            format!("Please specify single attribute as http_query, http_header, http_path, http_form_data or http_body."),
                        ));
                    }
                    result = Some(InputField::new(self, attr, HttpInputSource::Header));
                }
                HTTP_PATH_NAME => {
                    if result.is_some() {
                        return Err(syn::Error::new(
                            self.get_field_name_ident().span(),
                            format!("Please specify single attribute as http_query, http_header, http_path, http_form_data or http_body."),
                        ));
                    }
                    result = Some(InputField::new(self, attr, HttpInputSource::Path));
                }
                "http_form_data" => {
                    if result.is_some() {
                        return Err(syn::Error::new(
                            self.get_field_name_ident().span(),
                            format!("Please specify single attribute as http_query, http_header, http_path, http_form_data or http_body."),
                        ));
                    }
                    result = Some(InputField::new(self, attr, HttpInputSource::FormData));
                }
                "http_body" => {
                    if result.is_some() {
                        return Err(syn::Error::new(
                            self.get_field_name_ident().span(),
                            format!("Please specify single attribute as http_query, http_header, http_path, http_form_data or http_body."),
                        ));
                    }
                    result = Some(InputField::new(self, attr, HttpInputSource::Body));
                }
                "http_body_raw" => {
                    if result.is_some() {
                        return Err(syn::Error::new(
                            self.get_field_name_ident().span(),
                            format!("Please specify single attribute as http_query, http_header, http_path, http_form_data or http_body."),
                        ));
                    }
                    result = Some(InputField::new(self, attr, HttpInputSource::BodyRaw));
                }
                "ignore" => {
                    return Ok(None);
                }
                _ => {}
            }
        }

        if result.is_some() {
            return Ok(result);
        }

        return Err(syn::Error::new(
            self.get_field_name_ident().span(),
            format!("Please specify single attribute as http_query, http_header, http_path, http_form_data or http_body."),
        ));
    }

    fn get_struct_field_name_as_token_stream(&self) -> proc_macro2::TokenStream {
        let name = self.get_field_name_ident();
        quote::quote!(#name)
    }
}
