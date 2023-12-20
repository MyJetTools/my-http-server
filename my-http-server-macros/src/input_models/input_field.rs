use std::str::FromStr;

use proc_macro2::TokenStream;
use rust_extensions::StrOrString;
use types_reader::{PropertyType, StructProperty};

use super::HttpFieldAttribute;

#[derive(Clone)]
pub struct InputField<'s> {
    pub property: &'s StructProperty<'s>,
    pub attr: HttpFieldAttribute<'s>,
}

impl<'s> InputField<'s> {
    pub fn new<T: Into<HttpFieldAttribute<'s>>>(property: &'s StructProperty<'s>, attr: T) -> Self {
        Self {
            property,
            attr: attr.into(),
        }
    }

    pub fn get_input_field_name(&self) -> Result<&str, syn::Error> {
        if let Some(value) = self.attr.get_name() {
            Ok(value)
        } else {
            Ok(&self.property.name)
        }
    }

    fn is_str(&self) -> bool {
        match &self.property.ty {
            PropertyType::RefTo { ty, lifetime: _ } => ty.as_str().as_str() == "str",
            PropertyType::String => true,
            PropertyType::OptionOf(sub_ty) => match sub_ty.as_ref() {
                PropertyType::RefTo { ty, lifetime: _ } => ty.as_str().as_str() == "str",
                PropertyType::String => true,
                _ => false,
            },
            _ => false,
        }
    }

    pub fn has_default_value(&self) -> Result<bool, syn::Error> {
        Ok(self.attr.get_default().is_some())
    }

    pub fn to_lower_case_string(&self) -> Result<bool, syn::Error> {
        let result = self.attr.has_to_lower_case_attribute();

        if result && !self.is_str() {
            return self
                .property
                .throw_error("to_lowercase attribute can be only with String property");
        }

        Ok(result)
    }

    pub fn to_upper_case_string(&self) -> Result<bool, syn::Error> {
        let result = self.attr.has_to_upper_case_attribute();

        if result && !self.is_str() {
            return self
                .property
                .throw_error("to_uppercase attribute can be only with String property");
        }

        Ok(result)
    }

    pub fn read_value_with_transformation(&self) -> Result<TokenStream, syn::Error> {
        let ident = self.property.get_field_name_ident();

        let trim = if self.attr.has_trim_attribute() {
            quote::quote!(.trim())
        } else {
            quote::quote!()
        };

        if self.to_upper_case_string()? {
            if self.property.ty.is_option() {
                return Ok(
                    quote::quote!(#ident: if let Some(value) = #ident {value #trim .to_uppercase().into()}else{None}),
                );
            } else {
                return Ok(quote::quote!(#ident: #ident #trim .to_uppercase()));
            }
        }

        if self.to_lower_case_string()? {
            if self.property.ty.is_option() {
                return Ok(
                    quote::quote!(#ident: if let Some(value) = #ident {value #trim .to_lowercase().into()}else{None}),
                );
            } else {
                return Ok(quote::quote!(#ident: #ident #trim .to_lowercase()));
            }
        }

        if self.attr.has_trim_attribute() {
            if self.property.ty.is_option() {
                Ok(
                    quote::quote!(#ident: if let Some(value) = #ident {value.trim().to_string().into()}else{None}),
                )
            } else {
                Ok(quote::quote!(#ident: #ident.trim().to_string()))
            }
        } else {
            Ok(quote::quote!(#ident))
        }
    }

    pub fn get_default_value_opt_case(&'s self) -> Result<TokenStream, syn::Error> {
        let default_value = self.attr.get_default();

        if default_value.is_none() {
            return Ok(quote::quote!(None));
        }

        let default_value = default_value.unwrap();

        if default_value.has_empty_value() {
            let name = self.property.ty.get_token_stream();
            return Ok(quote::quote!(#name::create_default()?));
        }

        if let PropertyType::OptionOf(pt) = &self.property.ty {
            match pt.as_ref() {
                PropertyType::U8 => {
                    let value = default_value.get_value().unwrap_as_number()?.as_u8();
                    return Ok(quote::quote!(Some(#value)));
                }
                PropertyType::I8 => {
                    let value: i8 = default_value.get_value().unwrap_as_number()?.as_i8();
                    return Ok(quote::quote!(Some(#value)));
                }
                PropertyType::U16 => {
                    let value: u16 = default_value.get_value().unwrap_as_number()?.as_u16();
                    return Ok(quote::quote!(Some(#value)));
                }
                PropertyType::I16 => {
                    let value: i16 = default_value.get_value().unwrap_as_number()?.as_i16();
                    return Ok(quote::quote!(Some(#value)));
                }
                PropertyType::U32 => {
                    let value: u32 = default_value.get_value().unwrap_as_number()?.as_u32();
                    return Ok(quote::quote!(Some(#value)));
                }
                PropertyType::I32 => {
                    let value: i32 = default_value.get_value().unwrap_as_number()?.as_i32();
                    return Ok(quote::quote!(Some(#value)));
                }
                PropertyType::U64 => {
                    let value: u64 = default_value.get_value().unwrap_as_number()?.as_u64();
                    return Ok(quote::quote!(Some(#value)));
                }
                PropertyType::I64 => {
                    let value: i64 = default_value.get_value().unwrap_as_number()?.as_i64();
                    return Ok(quote::quote!(Some(#value)));
                }
                PropertyType::F32 => {
                    let value: f32 = default_value.get_value().unwrap_as_double()?.as_f32();
                    return Ok(quote::quote!(Some(#value)));
                }
                PropertyType::F64 => {
                    let value: f64 = default_value.get_value().unwrap_as_double()?.as_f64();
                    return Ok(quote::quote!(Some(#value)));
                }
                PropertyType::USize => {
                    let value: usize = default_value.get_value().unwrap_as_number()?.as_usize();
                    return Ok(quote::quote!(Some(#value)));
                }
                PropertyType::ISize => {
                    let value: isize = default_value.get_value().unwrap_as_number()?.as_isize();
                    return Ok(quote::quote!(Some(#value)));
                }
                PropertyType::String => {
                    let value: &str = default_value.get_value().unwrap_as_string()?.as_str();
                    return Ok(quote::quote!(Some(#value.to_string())));
                }

                PropertyType::Bool => {
                    let value: bool = default_value.get_value().unwrap_as_bool()?.get_value();
                    return Ok(quote::quote!(Some(#value)));
                }
                PropertyType::DateTime => {
                    let value: &str = default_value.get_value().unwrap_as_string()?.as_str();
                    return Ok(quote::quote!(Some(DateTimeAsMicroseconds::from_str(#value))));
                }
                PropertyType::OptionOf(_) => {
                    return Ok(quote::quote!(None));
                }
                PropertyType::VecOf(_) => {
                    return Ok(quote::quote!(None));
                }
                PropertyType::Struct(_, _) => {
                    return Ok(quote::quote!(None));
                }
                PropertyType::HashMap(_, _) => {
                    return Ok(quote::quote!(None));
                }
                PropertyType::RefTo { ty, lifetime: _ } => {
                    if ty.as_str().as_str() == "str" {
                        let value = default_value.get_value().unwrap_as_string()?.as_str();
                        return Ok(quote::quote!(Some(#value)));
                    }
                }
            }
        }

        return Ok(quote::quote!(None));
    }

    pub fn get_default_value_non_opt_case(&self) -> Result<TokenStream, syn::Error> {
        let default_value = self.attr.get_default();

        if default_value.is_none() {
            return Ok(quote::quote!(None));
        }

        let default_value = default_value.unwrap();

        if default_value.has_empty_value() {
            let name = self.property.ty.get_token_stream();
            return Ok(quote::quote!(#name::create_default()?));
        }

        match &self.property.ty {
            PropertyType::U8 => {
                let value: u8 = default_value.get_value().unwrap_as_number()?.as_u8();
                return Ok(quote::quote!(#value));
            }
            PropertyType::I8 => {
                let value: i8 = default_value.get_value().unwrap_as_number()?.as_i8();
                return Ok(quote::quote!(#value));
            }
            PropertyType::U16 => {
                let value: u16 = default_value.get_value().unwrap_as_number()?.as_u16();
                return Ok(quote::quote!(#value));
            }
            PropertyType::I16 => {
                let value: i16 = default_value.get_value().unwrap_as_number()?.as_i16();
                return Ok(quote::quote!(#value));
            }
            PropertyType::U32 => {
                let value: u32 = default_value.get_value().unwrap_as_number()?.as_u32();
                return Ok(quote::quote!(#value));
            }
            PropertyType::I32 => {
                let value: i32 = default_value.get_value().unwrap_as_number()?.as_i32();
                return Ok(quote::quote!(#value));
            }
            PropertyType::U64 => {
                let value: u64 = default_value.get_value().unwrap_as_number()?.as_u64();
                return Ok(quote::quote!(#value));
            }
            PropertyType::I64 => {
                let value: i64 = default_value.get_value().unwrap_as_number()?.as_i64();
                return Ok(quote::quote!(#value));
            }
            PropertyType::F32 => {
                let value: f32 = default_value.get_value().unwrap_as_double()?.as_f32();
                return Ok(quote::quote!(#value));
            }
            PropertyType::F64 => {
                let value: f64 = default_value.get_value().unwrap_as_double()?.as_f64();
                return Ok(quote::quote!(#value));
            }
            PropertyType::USize => {
                let value: usize = default_value.get_value().unwrap_as_number()?.as_usize();
                return Ok(quote::quote!(#value));
            }
            PropertyType::ISize => {
                let value: isize = default_value.get_value().unwrap_as_number()?.as_isize();
                return Ok(quote::quote!(#value));
            }
            PropertyType::String => {
                let value = default_value.get_value().unwrap_as_string()?.as_str();
                return Ok(quote::quote!(#value.to_string()));
            }
            PropertyType::Bool => {
                let value: bool = default_value.get_value().unwrap_as_bool()?.get_value();
                return Ok(quote::quote!(#value));
            }
            PropertyType::DateTime => {
                let value = default_value.get_value().unwrap_as_string()?.as_str();
                return Ok(quote::quote!(DateTimeAsMicroseconds::from_str(#value)));
            }
            PropertyType::OptionOf(_) => {
                return self.throw_error("Option default value is not supported");
            }
            PropertyType::VecOf(_) => {
                return self.throw_error("VecOf default value is not supported");
            }
            PropertyType::Struct(name, _) => {
                let name = TokenStream::from_str(name)?;
                return Ok(quote::quote!(#name::create_default()?));
            }
            PropertyType::HashMap(_, _) => {
                return self.throw_error("HashMap default value is not supported");
            }
            PropertyType::RefTo { ty, lifetime: _ } => {
                if ty.as_str().as_str() == "str" {
                    let value = default_value.get_value().unwrap_as_string()?.as_str();
                    return Ok(quote::quote!(#value));
                }

                return self.throw_error("Not supported type");
            }
        }
    }

    pub fn get_description(&self) -> &str {
        self.attr.description()
    }

    fn get_validator(&self) -> Option<&str> {
        self.attr.validator()
    }

    pub fn get_validator_as_token_stream(&self) -> Option<proc_macro2::TokenStream> {
        if let Some(validator) = self.get_validator() {
            let validation_fn_name = proc_macro2::TokenStream::from_str(validator).unwrap();
            let struct_field_name = self.property.get_field_name_ident();
            return Some(quote::quote!(#validation_fn_name(ctx, &#struct_field_name)?;));
        }

        None
    }

    pub fn get_let_input_param(&self) -> proc_macro2::TokenStream {
        match &self.property.ty {
            PropertyType::String => {
                let struct_name = self.property.get_field_name_ident();
                return quote::quote! {#struct_name: String};
            }
            PropertyType::OptionOf(sub_ty) => match sub_ty.as_ref() {
                PropertyType::String => {
                    let struct_name = self.property.get_field_name_ident();
                    return quote::quote! {#struct_name: Option<String>};
                }
                _ => {
                    let struct_name = self.property.get_field_name_ident();
                    return quote::quote! {#struct_name};
                }
            },
            _ => {
                let struct_name = self.property.get_field_name_ident();
                return quote::quote! {#struct_name};
            }
        }
    }

    pub fn throw_error<TResult>(
        &self,
        message: impl Into<StrOrString<'s>>,
    ) -> Result<TResult, syn::Error> {
        let message: StrOrString<'s> = message.into();
        let err = syn::Error::new_spanned(self.property.field, message.as_str());
        Err(err)
    }
}
