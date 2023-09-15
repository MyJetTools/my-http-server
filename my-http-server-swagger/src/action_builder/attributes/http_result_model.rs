use types_reader::ParamsList;

pub enum HttpResultModel {
    Object(String),
    Array(String),
    ArrayOfSimpleType(String),
    SimpleType(String),
}

impl HttpResultModel {
    pub fn new(param_list: &ParamsList) -> Result<Option<Self>, syn::Error> {
        match param_list.try_get_named_param("model") {
            Some(result) => Ok(Some(Self::create(
                result.unwrap_as_string_value()?.as_str(),
            ))),
            None => Ok(None),
        }
    }

    fn create(model: &str) -> Self {
        if let Some(vec_model) = is_model_vec(model) {
            if is_simple_type(vec_model) {
                return Self::ArrayOfSimpleType(vec_model.to_string()).into();
            }

            return Self::Array(vec_model.to_string()).into();
        }

        if is_simple_type(model) {
            return Self::SimpleType(model.to_string()).into();
        }

        return Self::Object(model.to_string()).into();
    }
}

fn is_simple_type(src: &str) -> bool {
    match src {
        "String" => true,
        "Integer" => true,
        "Long" => true,
        "Float" => true,
        "Double" => true,
        "Byte" => true,
        "Binary" => false,
        "Boolean" => true,
        "Date" => true,
        "DateTime" => true,
        "Password" => true,
        _ => false,
    }
}

fn is_model_vec(model_as_string: &str) -> Option<&str> {
    if model_as_string.starts_with("Vec<") {
        return Some(&model_as_string[4..model_as_string.len() - 1]);
    }

    None
}

#[cfg(test)]
mod tests {

    #[test]
    fn test_detecting_vec() {
        let src = "Vec<String>";
        assert_eq!(super::is_model_vec(src), Some("String"));
    }
}
