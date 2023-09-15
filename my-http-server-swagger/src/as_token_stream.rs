use proc_macro2::TokenStream;
use quote::quote;
use types_reader::{ParamValue, StructProperty};

use crate::input_models::InputField;

pub trait AsTokenStream {
    fn as_token_stream(&self) -> Result<TokenStream, syn::Error>;
}

impl<'s> AsTokenStream for Option<&'s ParamValue> {
    fn as_token_stream(&self) -> Result<TokenStream, syn::Error> {
        if let Some(value) = self {
            let value = value.unwrap_as_string_value()?.as_str();
            Ok(quote! {
                Some(#value)
            })
        } else {
            Ok(quote! {
                None
            })
        }
    }
}

/*
impl<'s> AsTokenStream for Option<DefaultValue<'s>> {
    fn as_token_stream(&self) -> Result<TokenStream, syn::Error> {
        if let Some(value) = self {
            let value = value.unwrap_value()?;
            Ok(quote! {
                Some(#value)
            })
        } else {
            Ok(quote! {
                None
            })
        }
    }
}

 */

impl<'s> AsTokenStream for Option<&'s str> {
    fn as_token_stream(&self) -> Result<TokenStream, syn::Error> {
        if let Some(value) = self {
            Ok(quote! {
                Some(#value)
            })
        } else {
            Ok(quote! {
                None
            })
        }
    }
}

impl<'s> AsTokenStream for Vec<&'s StructProperty<'s>> {
    fn as_token_stream(&self) -> Result<TokenStream, syn::Error> {
        if self.len() == 1 {
            let name = self.get(0).unwrap().get_field_name_ident();
            return Ok(quote!(#name));
        }

        let mut no = 0;

        let mut result = Vec::with_capacity(self.len());

        for input_field in self {
            if no > 0 {
                result.push(quote!(,));
            }

            let ident = input_field.get_field_name_ident();
            result.push(quote!(#ident));
            no += 1;
        }

        Ok(quote! {(#(#result)*)})
    }
}

impl<'s> AsTokenStream for Vec<&'s InputField<'s>> {
    fn as_token_stream(&self) -> Result<TokenStream, syn::Error> {
        if self.len() == 1 {
            let name = self.get(0).unwrap().property.get_field_name_ident();
            return Ok(quote!(#name));
        }

        let mut no = 0;

        let mut result = Vec::with_capacity(self.len());

        for input_field in self {
            if no > 0 {
                result.push(quote!(,));
            }

            let ident = input_field.property.get_field_name_ident();
            result.push(quote!(#ident));
            no += 1;
        }

        Ok(quote! {(#(#result)*)})
    }
}
