use super::HttpSimpleType;

pub enum ArrayElement {
    SimpleType(HttpSimpleType),
    ObjectId { struct_id: String },
}
