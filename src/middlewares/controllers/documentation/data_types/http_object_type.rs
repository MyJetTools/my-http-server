use super::{HttpDataType, HttpField};

pub struct HttpObjectType {
    pub struct_id: String,
    pub fields: Vec<HttpField>,
}

impl HttpObjectType {
    pub fn new(struct_id: &str) -> Self {
        Self {
            struct_id: struct_id.to_string(),
            fields: vec![],
        }
    }

    pub fn with_field(mut self, property: HttpField) -> Self {
        self.fields.push(property);
        self
    }

    pub fn with_string_field(mut self, name: &str, required: bool) -> Self {
        self.fields
            .push(HttpField::new(name, HttpDataType::as_string(), required));
        self
    }

    pub fn with_integer_field(mut self, name: &str, required: bool) -> Self {
        self.fields
            .push(HttpField::new(name, HttpDataType::as_integer(), required));
        self
    }

    pub fn with_long_field(mut self, name: &str, required: bool) -> Self {
        self.fields
            .push(HttpField::new(name, HttpDataType::as_long(), required));
        self
    }

    pub fn with_float_field(mut self, name: &str, required: bool) -> Self {
        self.fields
            .push(HttpField::new(name, HttpDataType::as_float(), required));
        self
    }

    pub fn with_double_field(mut self, name: &str, required: bool) -> Self {
        self.fields
            .push(HttpField::new(name, HttpDataType::as_double(), required));
        self
    }

    pub fn with_binary_field(mut self, name: &str, required: bool) -> Self {
        self.fields
            .push(HttpField::new(name, HttpDataType::as_binary(), required));
        self
    }

    pub fn with_date_field(mut self, name: &str, required: bool) -> Self {
        self.fields
            .push(HttpField::new(name, HttpDataType::as_date(), required));
        self
    }

    pub fn with_date_time_field(mut self, name: &str, required: bool) -> Self {
        self.fields
            .push(HttpField::new(name, HttpDataType::as_date_time(), required));
        self
    }

    pub fn with_password_field(mut self, name: &str, required: bool) -> Self {
        self.fields
            .push(HttpField::new(name, HttpDataType::as_password(), required));
        self
    }
}
