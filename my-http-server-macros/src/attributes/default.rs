use types_reader::macros::*;

#[attribute_name("default")]
#[derive(MacrosParameters, Clone)]
pub struct DefaultAttribute<'s> {
    #[default]
    pub value: types_reader::AnyValue<'s>,
}
