use super::HttpSimpleType;

pub enum ArrayElement {
    SimpleType(HttpSimpleType),
    Object { struct_id: String },
}
