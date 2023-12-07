use std::str::FromStr;

use proc_macro2::TokenStream;
use rust_extensions::StrOrString;
use types_reader::{ObjectValue, PropertyType, StructProperty, TokensObject};

#[derive(Clone)]
pub enum HttpInputSource {
    Query,
    Header,
    FormData,
    Body,
    BodyRaw,
    Path,
}

impl HttpInputSource {
    pub fn get_input_src_token(&self) -> proc_macro2::TokenStream {
        let http_parameter_input_src = crate::consts::get_http_parameter_input_src();
        match self {
            Self::Query => quote::quote!(#http_parameter_input_src::Query),
            Self::Path => quote::quote!(#http_parameter_input_src::Path),
            Self::Header => quote::quote!(#http_parameter_input_src::Header),
            Self::Body => quote::quote!(#http_parameter_input_src::BodyModel),
            Self::FormData => quote::quote!(#http_parameter_input_src::FormData),
            Self::BodyRaw => quote:: quote!(#http_parameter_input_src::BodyRaw),
        }
    }

    pub fn is_form_data(&self) -> bool {
        matches!(self, Self::FormData)
    }
}

pub enum DefaultValue<'s> {
    Empty(&'s TokensObject),
    Value(&'s TokensObject),
}

impl<'s> DefaultValue<'s> {
    pub fn unwrap_value(&'s self) -> Result<&'s ObjectValue, syn::Error> {
        match self {
            DefaultValue::Empty(value) => {
                Err(value.throw_error_at_value_token("Default value is not specified"))
            }
            DefaultValue::Value(value) => Ok(value.get_value()?),
        }
    }

    pub fn has_value(&self) -> bool {
        match self {
            DefaultValue::Empty(_) => false,
            DefaultValue::Value(_) => true,
        }
    }

    pub fn is_empty(&self) -> bool {
        match self {
            DefaultValue::Empty(_) => true,
            DefaultValue::Value(_) => false,
        }
    }

    pub fn throw_error<TOk>(
        &self,
        src: impl Into<StrOrString<'static>>,
    ) -> Result<TOk, syn::Error> {
        let src: StrOrString<'static> = src.into();
        match self {
            DefaultValue::Empty(value) => Err(value.throw_error_at_value_token(src.as_str())),
            DefaultValue::Value(value) => Err(value.throw_error_at_value_token(src.as_str())),
        }
    }
}

#[derive(Clone)]
pub struct InputField<'s> {
    pub property: &'s StructProperty<'s>,
    pub attr_params: &'s TokensObject,
    pub src: HttpInputSource,
}

impl<'s> InputField<'s> {
    pub fn new(
        property: &'s StructProperty<'s>,
        attr_params: &'s TokensObject,
        src: HttpInputSource,
    ) -> Self {
        Self {
            property,
            attr_params,
            src,
        }
    }

    pub fn get_input_field_name(&self) -> Result<&str, syn::Error> {
        if let Some(value) = self.attr_params.try_get_named_param("name") {
            Ok(value.get_value()?.as_string()?.into())
        } else {
            Ok(&self.property.name)
        }
    }

    pub fn has_default_value(&self) -> bool {
        self.attr_params.try_get_named_param("default").is_some()
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

    pub fn to_lower_case_string(&self) -> Result<bool, syn::Error> {
        let result = self
            .attr_params
            .try_get_named_param("to_lowercase")
            .is_some();

        if result && !self.is_str() {
            return self
                .property
                .throw_error("to_lowercase attribute can be only with String property");
        }

        Ok(result)
    }

    pub fn to_upper_case_string(&self) -> Result<bool, syn::Error> {
        let result = self
            .attr_params
            .try_get_named_param("to_uppercase")
            .is_some();

        if result && !self.is_str() {
            return self
                .property
                .throw_error("to_uppercase attribute can be only with String property");
        }

        Ok(result)
    }

    pub fn read_value_with_transformation(&self) -> Result<TokenStream, syn::Error> {
        let ident = self.property.get_field_name_ident();
        if self.to_upper_case_string()? {
            if self.property.ty.is_option() {
                return Ok(
                    quote::quote!(#ident: if let Some(value) = #ident {value.to_uppercase().into()}else{None}),
                );
            } else {
                return Ok(quote::quote!(#ident: #ident.to_uppercase()));
            }
        }

        if self.to_lower_case_string()? {
            if self.property.ty.is_option() {
                return Ok(
                    quote::quote!(#ident: if let Some(value) = #ident {value.to_lowercase().into()}else{None}),
                );
            } else {
                return Ok(quote::quote!(#ident: #ident.to_lowercase()));
            }
        }

        Ok(quote::quote!(#ident))
    }

    pub fn get_default_value(&self) -> Result<Option<DefaultValue>, syn::Error> {
        match self.attr_params.try_get_named_param("default") {
            Some(value) => {
                if value.has_no_value() {
                    return Ok(Some(DefaultValue::Empty(value)));
                }

                Ok(Some(DefaultValue::Value(value)))
            }
            None => Ok(None),
        }
    }

    pub fn get_default_value_opt_case(&self) -> Result<TokenStream, syn::Error> {
        if let Some(default) = self.get_default_value()? {
            if default.is_empty() {
                let name = self.property.ty.get_token_stream();
                return Ok(quote::quote!(#name::create_default()?));
            }

            let value = default.unwrap_value()?;
            if let PropertyType::OptionOf(pt) = &self.property.ty {
                match pt.as_ref() {
                    PropertyType::U8 => {
                        let value = value.as_number()?.as_u8();
                        return Ok(quote::quote!(Some(#value)));
                    }
                    PropertyType::I8 => {
                        let value = value.as_number()?.as_i8();
                        return Ok(quote::quote!(Some(#value)));
                    }
                    PropertyType::U16 => {
                        let value = value.as_number()?.as_u16();
                        return Ok(quote::quote!(Some(#value)));
                    }
                    PropertyType::I16 => {
                        let value = value.as_number()?.as_i16();
                        return Ok(quote::quote!(Some(#value)));
                    }
                    PropertyType::U32 => {
                        let value = value.as_number()?.as_u32();
                        return Ok(quote::quote!(Some(#value)));
                    }
                    PropertyType::I32 => {
                        let value = value.as_number()?.as_i32();
                        return Ok(quote::quote!(Some(#value)));
                    }
                    PropertyType::U64 => {
                        let value = value.as_number()?.as_u64();
                        return Ok(quote::quote!(Some(#value)));
                    }
                    PropertyType::I64 => {
                        let value = value.as_number()?.as_i64();
                        return Ok(quote::quote!(Some(#value)));
                    }
                    PropertyType::F32 => {
                        let value = value.as_double()?.as_f32();
                        return Ok(quote::quote!(Some(#value)));
                    }
                    PropertyType::F64 => {
                        let value = value.as_double()?.as_f64();
                        return Ok(quote::quote!(Some(#value)));
                    }
                    PropertyType::USize => {
                        let value = value.as_number()?.as_i64() as usize;
                        return Ok(quote::quote!(Some(#value)));
                    }
                    PropertyType::ISize => {
                        let value = value.as_number()?.as_i64() as isize;
                        return Ok(quote::quote!(Some(#value)));
                    }
                    PropertyType::String => {
                        let value = value.as_string()?.as_str();
                        return Ok(quote::quote!(Some(#value.to_string())));
                    }

                    PropertyType::Bool => {
                        let value = value.as_bool()?.get_value();
                        return Ok(quote::quote!(Some(#value)));
                    }
                    PropertyType::DateTime => {
                        let value = value.as_string()?;
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
                            let value = value.as_string()?.as_str();
                            return Ok(quote::quote!(Some(#value)));
                        }
                    }
                }
            }
        }

        return Ok(quote::quote!(None));
    }

    pub fn get_default_value_non_opt_case(&self) -> Result<TokenStream, syn::Error> {
        if let Some(default) = self.get_default_value()? {
            if default.is_empty() {
                let name = self.property.ty.get_token_stream();
                return Ok(quote::quote!(#name::create_default()?));
            }

            let value = default.unwrap_value()?;

            match &self.property.ty {
                PropertyType::U8 => {
                    let value = value.as_number()?.as_u8();
                    return Ok(quote::quote!(#value));
                }
                PropertyType::I8 => {
                    let value = value.as_number()?.as_i8();
                    return Ok(quote::quote!(#value));
                }
                PropertyType::U16 => {
                    let value = value.as_number()?.as_u16();
                    return Ok(quote::quote!(#value));
                }
                PropertyType::I16 => {
                    let value = value.as_number()?.as_i16();
                    return Ok(quote::quote!(#value));
                }
                PropertyType::U32 => {
                    let value = value.as_number()?.as_u32();
                    return Ok(quote::quote!(#value));
                }
                PropertyType::I32 => {
                    let value = value.as_number()?.as_i32();
                    return Ok(quote::quote!(#value));
                }
                PropertyType::U64 => {
                    let value = value.as_number()?.as_u64();
                    return Ok(quote::quote!(#value));
                }
                PropertyType::I64 => {
                    let value = value.as_number()?.as_i64();
                    return Ok(quote::quote!(#value));
                }
                PropertyType::F32 => {
                    let value = value.as_double()?.as_f32();
                    return Ok(quote::quote!(#value));
                }
                PropertyType::F64 => {
                    let value = value.as_double()?.as_f64();
                    return Ok(quote::quote!(#value));
                }
                PropertyType::USize => {
                    let value = value.as_number()?.as_i64() as usize;
                    return Ok(quote::quote!(#value));
                }
                PropertyType::ISize => {
                    let value = value.as_number()?.as_i64() as isize;
                    return Ok(quote::quote!(#value));
                }
                PropertyType::String => {
                    let value = value.as_string()?.as_str();
                    return Ok(quote::quote!(#value.to_string()));
                }
                PropertyType::Bool => {
                    let value = value.as_bool()?.get_value();
                    return Ok(quote::quote!(#value));
                }
                PropertyType::DateTime => {
                    let value = value.as_string()?;
                    return Ok(quote::quote!(DateTimeAsMicroseconds::from_str(#value)));
                }
                PropertyType::OptionOf(_) => {
                    return Err(value.throw_error("Option default value is not supported"));
                }
                PropertyType::VecOf(_) => {
                    return Err(value.throw_error("VecOf default value is not supported"));
                }
                PropertyType::Struct(name, _) => {
                    let name = TokenStream::from_str(name)?;
                    return Ok(quote::quote!(#name::create_default()?));
                }
                PropertyType::HashMap(_, _) => {
                    return Err(value.throw_error("HashMap default value is not supported"));
                }
                PropertyType::RefTo { ty, lifetime: _ } => {
                    if ty.as_str().as_str() == "str" {
                        let value = value.as_string()?.as_str();
                        return Ok(quote::quote!(#value));
                    }

                    return Err(value.throw_error(
                        format!(
                            "Ref of {} default value is not supported",
                            ty.as_str().as_str(),
                        )
                        .as_str(),
                    ));
                }
            }
        }

        return Ok(quote::quote!(None));
    }

    pub fn get_description(&self) -> Result<&str, syn::Error> {
        let result = self.attr_params.get_named_param("description")?;
        Ok(result.get_value()?.as_string()?.as_str())
    }

    pub fn validator(&self) -> Result<Option<&str>, syn::Error> {
        let result = self.attr_params.try_get_named_param("validator");

        match result {
            Some(value) => Ok(Some(&value.get_value()?.any_value_as_str().as_str())),
            _ => Ok(None),
        }
    }

    pub fn get_validator(&self) -> Result<Option<proc_macro2::TokenStream>, syn::Error> {
        if let Some(validator) = self.validator()? {
            let validation_fn_name = proc_macro2::TokenStream::from_str(validator).unwrap();
            let struct_field_name = self.property.get_field_name_ident();
            return Ok(Some(
                quote::quote!(#validation_fn_name(ctx, &#struct_field_name)?;),
            ));
        }

        Ok(None)
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
