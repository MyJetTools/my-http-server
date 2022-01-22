pub struct JsonObjectWriter {
    first_element: bool,
    raw: Vec<u8>,
    last_element: u8,
}

impl JsonObjectWriter {
    pub fn as_object() -> Self {
        let mut raw = Vec::new();
        raw.push('{' as u8);
        Self {
            raw,
            first_element: true,
            last_element: '}' as u8,
        }
    }

    pub fn has_written(&self) -> bool {
        !self.first_element
    }

    pub fn as_array() -> Self {
        let mut raw = Vec::new();
        raw.push('[' as u8);
        Self {
            raw,
            first_element: true,
            last_element: ']' as u8,
        }
    }

    fn add_delimetr(&mut self) {
        if self.first_element {
            self.first_element = false;
        } else {
            self.raw.push(',' as u8);
        }
    }

    pub fn write_string_element(&mut self, value: &str) {
        self.add_delimetr();
        let data_to_add = format!("\"{}\"", value);
        self.raw.extend(data_to_add.into_bytes());
    }

    pub fn write_number_element(&mut self, value: String) {
        self.add_delimetr();

        self.raw.extend(value.into_bytes());
    }

    pub fn write_array_object_element(&mut self, object: JsonObjectWriter) {
        self.add_delimetr();
        self.raw.extend(object.build());
    }

    pub fn write_string_value(&mut self, key: &str, value: &str) {
        self.add_delimetr();
        let data_to_add = format!("\"{}\":\"{}\"", key, value);
        self.raw.extend(data_to_add.into_bytes());
    }

    pub fn write_bool_value(&mut self, key: &str, value: bool) {
        self.add_delimetr();
        let data_to_add = format!("\"{}\":\"{}\"", key, value);
        self.raw.extend(data_to_add.into_bytes());
    }

    pub fn write_object(&mut self, key: &str, object: JsonObjectWriter) {
        self.add_delimetr();
        let data_to_add = format!("\"{}\":", key);

        self.raw.extend(data_to_add.into_bytes());
        self.raw.extend(object.build());
    }

    pub fn build(mut self) -> Vec<u8> {
        self.raw.push(self.last_element);
        self.raw
    }
}
