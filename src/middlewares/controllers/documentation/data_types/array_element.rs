use super::{HttpObjectType, HttpSimpleType};

pub enum ArrayElement {
    SimpleType(HttpSimpleType),
    ObjectId { struct_id: String },
    Object(HttpObjectType),
}
