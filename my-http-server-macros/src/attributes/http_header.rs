use types_reader::macros::*;

#[attribute_name("http_header")]
#[derive(MacrosParameters, Clone)]
pub struct HttpHeaderAttribute<'s> {
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
