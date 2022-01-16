use super::{HttpSimpleType};

pub enum ArrayElement {
    SimpleType(HttpSimpleType),
    Object(String),
}
