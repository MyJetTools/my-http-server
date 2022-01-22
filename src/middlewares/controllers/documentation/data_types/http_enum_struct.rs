use std::collections::BTreeMap;

pub struct HttpEnumCase {
    pub value: String,
    pub description: String,
}

pub struct HttpEnumStructure {
    pub struct_id: String,
    pub cases: BTreeMap<usize, HttpEnumCase>,
}
