pub enum SwaggerParameterType {
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

impl SwaggerParameterType {
    pub fn to_str(&self) -> &str {
        match self {
            SwaggerParameterType::Integer => "integer",
            SwaggerParameterType::Long => "long",
            SwaggerParameterType::Float => "float",
            SwaggerParameterType::Double => "double",
            SwaggerParameterType::String => "string",
            SwaggerParameterType::Byte => "byte",
            SwaggerParameterType::Binary => "binary",
            SwaggerParameterType::Boolean => "boolean",
            SwaggerParameterType::Date => "date",
            SwaggerParameterType::DateTime => "dateTime",
            SwaggerParameterType::Password => "password",
        }
    }

    pub fn as_swagger_type(&self) -> &str {
        match self {
            SwaggerParameterType::Integer => "integer",
            SwaggerParameterType::Long => "integer",
            SwaggerParameterType::Float => "number",
            SwaggerParameterType::Double => "number",
            SwaggerParameterType::String => "string",
            SwaggerParameterType::Byte => "string",
            SwaggerParameterType::Binary => "string",
            SwaggerParameterType::Boolean => "boolean",
            SwaggerParameterType::Date => "string",
            SwaggerParameterType::DateTime => "string",
            SwaggerParameterType::Password => "string",
        }
    }
}
