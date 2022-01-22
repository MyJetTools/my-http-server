use super::HttpDataType;

pub struct HttpField {
    pub name: String,
    pub data_type: HttpDataType,
    pub required: bool,
    pub default_value: Option<String>,
}

impl HttpField {
    pub fn new(
        name: &str,
        data_type: HttpDataType,
        required: bool,
        default_value: Option<String>,
    ) -> Self {
        Self {
            name: name.to_string(),
            data_type,
            required,
            default_value,
        }
    }
}
