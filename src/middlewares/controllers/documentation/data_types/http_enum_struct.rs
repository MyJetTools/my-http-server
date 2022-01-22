use std::collections::BTreeMap;

pub struct HttpEnumCase {
    pub value: String,
    pub description: String,
}

pub enum EnumType {
    Integer,
    String,
}

pub struct HttpEnumStructure {
    pub struct_id: String,
    pub enum_type: EnumType,
    pub cases: BTreeMap<usize, HttpEnumCase>,
}
