use crate::controllers::documentation::{data_types::HttpField, ArrayElement, HttpSimpleType};

use super::HttpParameterInputSource;

pub enum NonBodyParameter {
    SimpleType(HttpSimpleType),
    ArrayOf(ArrayElement),
}

#[derive(Debug, Clone)]
pub struct HttpInputParameter {
    pub field: HttpField,
    pub description: String,
    pub source: HttpParameterInputSource,
}

impl HttpInputParameter {
    pub fn is_body_reader(&self) -> bool {
        match self.source {
            HttpParameterInputSource::BodyModel => {
                return true;
            }
            _ => false,
        }
    }

    pub fn is_form_data(&self) -> bool {
        match self.source {
            HttpParameterInputSource::FormData => {
                return true;
            }
            _ => false,
        }
    }

    pub fn is_file_to_upload_from_body(&self) -> bool {
        if self.field.is_file_upload() {
            match self.source {
                HttpParameterInputSource::BodyRaw => {
                    return true;
                }
                _ => {}
            }
        }

        false
    }
}
