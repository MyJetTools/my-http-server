use super::{HttpObjectDescription, HttpSimpleType};

pub enum ArrayElement {
    SimpleType(HttpSimpleType),
    Object(HttpObjectDescription),
}
