pub struct PasswordHttpInputField(String);

impl PasswordHttpInputField {
    pub fn new(value: String) -> Self {
        Self(value)
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn into_string(self) -> String {
        self.0
    }
}

impl crate::documentation::DataTypeProvider for PasswordHttpInputField {
    fn get_data_type() -> crate::documentation::HttpDataType {
        crate::documentation::HttpDataType::SimpleType(
            crate::documentation::HttpSimpleType::Password,
        )
    }
}

impl Into<String> for PasswordHttpInputField {
    fn into(self) -> String {
        self.0
    }
}

impl AsRef<str> for PasswordHttpInputField {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl Into<PasswordHttpInputField> for String {
    fn into(self) -> PasswordHttpInputField {
        PasswordHttpInputField::new(self)
    }
}

impl<'s> TryInto<PasswordHttpInputField> for my_http_server_core::HeaderValue<'s> {
    type Error = my_http_server_core::HttpFailResult;
    fn try_into(self) -> Result<PasswordHttpInputField, Self::Error> {
        let src = self.as_str()?;
        Ok(PasswordHttpInputField(src.to_string()))
    }
}

impl<'s> TryInto<PasswordHttpInputField> for my_http_server_core::EncodedParamValue<'s> {
    type Error = my_http_server_core::HttpFailResult;
    fn try_into(self) -> Result<PasswordHttpInputField, Self::Error> {
        let src = self.as_str()?;
        Ok(PasswordHttpInputField::new(src.to_string()))
    }
}
impl<'s> TryInto<PasswordHttpInputField> for my_http_server_core::FormDataItem<'s> {
    type Error = my_http_server_core::HttpFailResult;
    fn try_into(self) -> Result<PasswordHttpInputField, Self::Error> {
        match self {
            my_http_server_core::FormDataItem::ValueAsString { name, value } => match value {
                Some(value) => return Ok(PasswordHttpInputField::new(value.to_string())),
                None => {
                    return Err(
                        my_http_server_core::HttpFailResult::required_parameter_is_missing(
                            name, "FormData",
                        ),
                    )
                }
            },
            my_http_server_core::FormDataItem::File { name, .. } => {
                Err(my_http_server_core::HttpFailResult::as_validation_error(
                    format!("{name} Can not be read from FormData file",),
                ))
            }
        }
    }
}
