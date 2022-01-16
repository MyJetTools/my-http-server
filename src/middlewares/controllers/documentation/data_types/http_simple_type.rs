pub enum HttpSimpleType {
    Integer,
    Long,
    Float,
    Double,
    String,
    Byte,
    Binary,
    Boolean,
    Date,
    DateTime,
    Password,
}

impl HttpSimpleType {
    pub fn as_str(&self) -> &str {
        match self {
            HttpSimpleType::Integer => "integer",
            HttpSimpleType::Long => "long",
            HttpSimpleType::Float => "float",
            HttpSimpleType::Double => "double",
            HttpSimpleType::String => "string",
            HttpSimpleType::Byte => "byte",
            HttpSimpleType::Binary => "binary",
            HttpSimpleType::Boolean => "boolean",
            HttpSimpleType::Date => "date",
            HttpSimpleType::DateTime => "dateTime",
            HttpSimpleType::Password => "password",
        }
    }

    pub fn as_swagger_type(&self) -> &str {
        match self {
            HttpSimpleType::Integer => "integer",
            HttpSimpleType::Long => "integer",
            HttpSimpleType::Float => "number",
            HttpSimpleType::Double => "number",
            HttpSimpleType::String => "string",
            HttpSimpleType::Byte => "string",
            HttpSimpleType::Binary => "string",
            HttpSimpleType::Boolean => "boolean",
            HttpSimpleType::Date => "string",
            HttpSimpleType::DateTime => "string",
            HttpSimpleType::Password => "string",
        }
    }
}
