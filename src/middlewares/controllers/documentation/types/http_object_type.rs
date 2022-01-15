use super::HttpDataType;

pub struct HttpObjectProperty {
    pub name: String,
    pub prop_type: HttpDataType,
}

impl HttpObjectProperty {
    pub fn new(name: &str, prop_type: HttpDataType) -> Self {
        Self {
            name: name.to_string(),
            prop_type: prop_type,
        }
    }
}

pub struct HttpObjectDescription {
    pub struct_id: String,
    pub properties: Vec<HttpObjectProperty>,
}
