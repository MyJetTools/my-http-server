use std::{fmt::Debug, str::FromStr};

use rust_extensions::StrOrString;

use crate::attributes::DefaultAttribute;

pub struct DefaultValue<'s> {
    pub attr: DefaultAttribute<'s>,
}

impl<'s> DefaultValue<'s> {
    pub fn new(attr: DefaultAttribute<'s>) -> Self {
        Self { attr }
    }
    pub fn get_value<TResult: FromStr + Debug>(&'s self) -> Result<TResult, syn::Error> {
        let value = self.attr.value.unwrap_value()?;
        let result = value.any_value_as_str();

        match result.parse() {
            Ok(result) => return Ok(result),
            Err(_) => return Err(self.attr.value.throw_error("Can not parse value")),
        }
    }

    pub fn as_str(&self) -> Result<&str, syn::Error> {
        let value = self.attr.value.unwrap_value()?;
        let result = value.any_value_as_str();
        Ok(result)
    }

    pub fn has_value(&self) -> bool {
        !self.attr.value.has_no_value()
    }

    pub fn is_empty(&self) -> bool {
        self.attr.value.has_no_value()
    }

    pub fn throw_error<TOk>(
        &self,
        src: impl Into<StrOrString<'static>>,
    ) -> Result<TOk, syn::Error> {
        let src: StrOrString<'static> = src.into();
        Err(self.attr.value.throw_error(src.as_str()))
    }
}
