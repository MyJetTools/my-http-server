use rust_extensions::StrOrString;

use super::{ArrayElement, HttpDataType, HttpField};
#[derive(Clone, Debug)]
pub struct HttpObjectFields {
    pub struct_id: &'static str,
    pub fields: Vec<HttpField>,
}

#[derive(Clone, Debug)]
pub struct HttpObjectStructure {
    pub main: HttpObjectFields,
    pub generic: Option<HttpObjectFields>,
}

impl super::InputStructure for HttpObjectStructure {
    fn get_struct_id(&self) -> StrOrString<'static> {
        match &self.generic {
            Some(generic_data) => {
                format!("{}_{}", self.main.struct_id, generic_data.struct_id).into()
            }
            None => self.main.struct_id.into(),
        }
    }
}

impl HttpObjectStructure {
    pub fn into_http_data_type_object(self) -> HttpDataType {
        HttpDataType::Object(self)
    }

    pub fn into_http_data_type_array(self) -> HttpDataType {
        HttpDataType::ArrayOf(ArrayElement::Object(self))
    }

    pub fn new(struct_id: &'static str, generic_struct_id: Option<&'static str>) -> Self {
        let generic = if let Some(generic_struct_id) = generic_struct_id {
            Some(HttpObjectFields {
                struct_id: generic_struct_id,
                fields: vec![],
            })
        } else {
            None
        };

        Self {
            main: HttpObjectFields {
                struct_id,
                fields: vec![],
            },
            generic,
        }
    }
}
