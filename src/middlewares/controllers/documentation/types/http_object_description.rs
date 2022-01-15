use super::{HttpDataType, HttpObjectProperty};

pub struct HttpObjectDescription {
    pub struct_id: String,
    pub properties: Vec<HttpObjectProperty>,
}

impl HttpObjectDescription {
    pub fn new(struct_id: &str) -> Self {
        Self {
            struct_id: struct_id.to_string(),
            properties: vec![],
        }
    }

    pub fn with_property(mut self, property: HttpObjectProperty) -> Self {
        self.properties.push(property);
        self
    }

    pub fn with_string_property(mut self, name: &str, required: bool) -> Self {
        self.properties.push(HttpObjectProperty::new(
            name,
            HttpDataType::as_string(required),
        ));
        self
    }

    pub fn with_integer_property(mut self, name: &str, required: bool) -> Self {
        self.properties.push(HttpObjectProperty::new(
            name,
            HttpDataType::as_integer(required),
        ));
        self
    }

    pub fn with_long_property(mut self, name: &str, required: bool) -> Self {
        self.properties.push(HttpObjectProperty::new(
            name,
            HttpDataType::as_long(required),
        ));
        self
    }

    pub fn with_float_property(mut self, name: &str, required: bool) -> Self {
        self.properties.push(HttpObjectProperty::new(
            name,
            HttpDataType::as_float(required),
        ));
        self
    }

    pub fn with_double_property(mut self, name: &str, required: bool) -> Self {
        self.properties.push(HttpObjectProperty::new(
            name,
            HttpDataType::as_double(required),
        ));
        self
    }

    pub fn with_binary_property(mut self, name: &str, required: bool) -> Self {
        self.properties.push(HttpObjectProperty::new(
            name,
            HttpDataType::as_binary(required),
        ));
        self
    }

    pub fn with_date_property(mut self, name: &str, required: bool) -> Self {
        self.properties.push(HttpObjectProperty::new(
            name,
            HttpDataType::as_date(required),
        ));
        self
    }

    pub fn with_date_time_property(mut self, name: &str, required: bool) -> Self {
        self.properties.push(HttpObjectProperty::new(
            name,
            HttpDataType::as_date_time(required),
        ));
        self
    }

    pub fn with_password_property(mut self, name: &str, required: bool) -> Self {
        self.properties.push(HttpObjectProperty::new(
            name,
            HttpDataType::as_password(required),
        ));
        self
    }
}
