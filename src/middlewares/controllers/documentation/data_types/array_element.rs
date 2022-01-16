use super::{HttpObjectStructure, HttpSimpleType};

pub enum ArrayElement {
    SimpleType(HttpSimpleType),
    ObjectId { struct_id: String },
    Object(HttpObjectStructure),
}
