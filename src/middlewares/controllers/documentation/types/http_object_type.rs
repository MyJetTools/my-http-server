use super::HttpDataType;

pub struct HttpObjectProperty {
    pub name: String,
    pub prop_type: HttpDataType,
}

pub struct HttpObjectDescription {
    pub struct_id: String,
    pub properties: Vec<HttpObjectProperty>,
}
