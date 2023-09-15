use hyper::Method;
use rust_extensions::lazy::LazyVec;

use super::{HttpInputParameter, HttpParameterInputSource};

#[derive(Debug)]
pub struct HttpParameters {
    non_body_params: Option<Vec<HttpInputParameter>>,
    body_params: Option<Vec<HttpInputParameter>>,
    form_data_params: Option<Vec<HttpInputParameter>>,
}

impl HttpParameters {
    pub fn new(params: Option<Vec<HttpInputParameter>>) -> Self {
        if params.is_none() {
            return Self {
                non_body_params: None,
                body_params: None,
                form_data_params: None,
            };
        }

        let params = params.unwrap();

        let mut non_body_params = LazyVec::new();
        let mut body_params = LazyVec::new();
        let mut form_data_params = LazyVec::new();

        for param in params {
            if param.source.is_body() {
                body_params.add(param);
            } else if param.source.is_form_data() {
                form_data_params.add(param);
            } else {
                non_body_params.add(param);
            }
        }

        let result = Self {
            body_params: body_params.get_result(),
            non_body_params: non_body_params.get_result(),
            form_data_params: form_data_params.get_result(),
        };

        result
    }

    pub fn get_non_body_params(&self) -> Option<&Vec<HttpInputParameter>> {
        self.non_body_params.as_ref()
    }

    pub fn get_body_params(&self) -> Option<&Vec<HttpInputParameter>> {
        self.body_params.as_ref()
    }

    pub fn get_form_data_params(&self) -> Option<&Vec<HttpInputParameter>> {
        self.form_data_params.as_ref()
    }

    pub fn is_single_body_parameter(&self) -> Option<&HttpInputParameter> {
        let params = self.body_params.as_ref()?;

        if params.len() != 1 {
            return None;
        }

        let param = params.get(0).unwrap();

        match &param.source {
            HttpParameterInputSource::BodyRaw => Some(param),
            _ => None,
        }
    }

    pub fn check_parameters(&self, method: &Method, route: &str) {
        let mut found_body_param = None;
        if let Some(body_params) = &self.body_params {
            for body_param in body_params {
                found_body_param = Some(body_param);
                break;
            }
        }

        if let Some(form_data_params) = &self.form_data_params {
            for param in form_data_params {
                found_body_param = Some(param);
                break;
            }
        }

        if let Some(found_body_param) = found_body_param {
            match method {
                &Method::GET => {
                    if self.body_params.is_some() {
                        panic!(
                            "GET method cannot have body parameters. Please check param {} for route {}",
                            found_body_param.field.name,
                            route
                        );
                    }
                }
                &Method::DELETE => {
                    if self.body_params.is_some() {
                        panic!(
                            "DELETE method cannot have body parameters. Please check param {} for route {}",
                            found_body_param.field.name, route
                        );
                    }
                }
                _ => {}
            }
        }
    }
}

impl From<Vec<HttpInputParameter>> for HttpParameters {
    fn from(src: Vec<HttpInputParameter>) -> Self {
        if src.len() == 0 {
            return Self::new(None);
        } else {
            Self::new(Some(src))
        }
    }
}
