use super::HttpDataType;

pub struct HttpObjectProperty {
    pub name: String,
    pub prop_type: HttpDataType,
    pub required: bool,
}

impl HttpObjectProperty {
    pub fn new(name: &str, prop_type: HttpDataType, required: bool) -> Self {
        Self {
            name: name.to_string(),
            prop_type,
            required,
        }
    }
}
