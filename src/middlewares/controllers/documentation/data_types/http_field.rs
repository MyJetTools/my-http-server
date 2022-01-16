use super::HttpDataType;

pub struct HttpField {
    pub name: String,
    pub data_type: HttpDataType,
    pub required: bool,
}

impl HttpField {
    pub fn new(name: &str, data_type: HttpDataType, required: bool) -> Self {
        Self {
            name: name.to_string(),
            data_type,
            required,
        }
    }
}
