use std::str::FromStr;

use types_reader::StructProperty;

pub struct HttpInputDefaultValue<'s> {
    value: &'s str,
}

impl<'s> HttpInputDefaultValue<'s> {
    pub fn new(value: &'s str) -> Self {
        Self { value }
    }

    pub fn has_empty_value(&self) -> bool {
        self.value.is_empty()
    }

    pub fn get_str(&'s self) -> &str {
        self.value
    }

    pub fn get_value<TResult: FromStr>(
        &'s self,
        prop: &'s StructProperty<'s>,
    ) -> Result<TResult, syn::Error> {
        let value = self.get_str();

        match value.parse() {
            Ok(result) => Ok(result),
            Err(_) => {
                return Err(syn::Error::new_spanned(
                    prop.get_field_name_ident(),
                    format!("Invalid default value: {}", value),
                ))
            }
        }
    }
}
