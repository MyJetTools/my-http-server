use types_reader::{macros::*, OptionalObjectValue};

#[attribute_name("http_form_data")]
#[derive(MacrosParameters, Clone)]
pub struct HttpFormDataAttribute<'s> {
    pub name: Option<&'s str>,
    pub description: &'s str,

    #[allow_ident]
    pub validator: Option<&'s str>,

    #[has_attribute]
    pub to_lowercase: bool,

    #[has_attribute]
    pub to_uppercase: bool,

    #[has_attribute]
    pub trim: bool,

    #[any_value_as_string]
    pub default: Option<&'s str>,
}
