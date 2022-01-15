pub enum HttpParameterType {
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

impl HttpParameterType {
    pub fn as_str(&self) -> &str {
        match self {
            HttpParameterType::Integer => "integer",
            HttpParameterType::Long => "long",
            HttpParameterType::Float => "float",
            HttpParameterType::Double => "double",
            HttpParameterType::String => "string",
            HttpParameterType::Byte => "byte",
            HttpParameterType::Binary => "binary",
            HttpParameterType::Boolean => "boolean",
            HttpParameterType::Date => "date",
            HttpParameterType::DateTime => "dateTime",
            HttpParameterType::Password => "password",
        }
    }

    pub fn as_swagger_type(&self) -> &str {
        match self {
            HttpParameterType::Integer => "integer",
            HttpParameterType::Long => "integer",
            HttpParameterType::Float => "number",
            HttpParameterType::Double => "number",
            HttpParameterType::String => "string",
            HttpParameterType::Byte => "string",
            HttpParameterType::Binary => "string",
            HttpParameterType::Boolean => "boolean",
            HttpParameterType::Date => "string",
            HttpParameterType::DateTime => "string",
            HttpParameterType::Password => "string",
        }
    }
}
