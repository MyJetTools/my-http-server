use types_reader::{macros::*, OptionalObjectValue};

#[attribute_name("default")]
#[derive(MacrosParameters, Clone)]
pub struct DefaultAttribute<'s> {
    #[default]
    pub value: &'s OptionalObjectValue,
}
