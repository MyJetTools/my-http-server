pub struct HttpEnumCase {
    pub id: usize,
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
    pub cases: Vec<HttpEnumCase>,
}
