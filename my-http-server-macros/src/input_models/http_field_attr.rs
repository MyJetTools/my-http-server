use crate::attributes::*;

use super::HttpInputDefaultValue;

#[derive(Clone)]
pub enum HttpFieldAttribute<'s> {
    HttpHeader(HttpHeaderAttribute<'s>),
    HttpQuery(HttpQueryAttribute<'s>),
    HttpBody(HttpBodyAttribute<'s>),
    HttpFormData(HttpFormDataAttribute<'s>),
    HttpBodyRaw(HttpBodyRawAttribute<'s>),
    HttpPath(HttpPathAttribute<'s>),
}

impl<'s> HttpFieldAttribute<'s> {
    pub fn get_default(&'s self) -> Option<HttpInputDefaultValue<'s>> {
        let default_attr = match self {
            Self::HttpHeader(http_header) => http_header.default.clone(),
            Self::HttpQuery(http_query) => http_query.default.clone(),
            Self::HttpBody(http_body) => http_body.default.clone(),
            Self::HttpFormData(http_form_data) => http_form_data.default.clone(),
            Self::HttpBodyRaw(http_body_raw) => http_body_raw.default.clone(),
            Self::HttpPath(http_path) => http_path.default.clone(),
        };

        if default_attr.is_none() {
            return None;
        }
        let result = HttpInputDefaultValue::new(default_attr.unwrap());

        Some(result)
    }

    pub fn validator(&'s self) -> Option<&'s str> {
        match self {
            Self::HttpHeader(http_header) => http_header.validator,
            Self::HttpQuery(http_query) => http_query.validator,
            Self::HttpBody(http_body) => http_body.validator,
            Self::HttpFormData(http_form_data) => http_form_data.validator,
            Self::HttpBodyRaw(http_body_raw) => http_body_raw.validator,
            Self::HttpPath(http_path) => http_path.validator,
        }
    }

    pub fn get_name(&'s self) -> Option<&'s str> {
        match self {
            Self::HttpHeader(http_header) => http_header.name,
            Self::HttpQuery(http_query) => http_query.name,
            Self::HttpBody(http_body) => http_body.name,
            Self::HttpFormData(http_form_data) => http_form_data.name,
            Self::HttpBodyRaw(http_body_raw) => http_body_raw.name,
            Self::HttpPath(http_path) => http_path.name,
        }
    }

    pub fn has_to_lower_case_attribute(&self) -> bool {
        match self {
            Self::HttpHeader(http_header) => http_header.to_lowercase,
            Self::HttpQuery(http_query) => http_query.to_lowercase,
            Self::HttpBody(http_body) => http_body.to_lowercase,
            Self::HttpFormData(http_form_data) => http_form_data.to_lowercase,
            Self::HttpBodyRaw(http_body_raw) => http_body_raw.to_lowercase,
            Self::HttpPath(http_path) => http_path.to_lowercase,
        }
    }

    pub fn has_to_upper_case_attribute(&self) -> bool {
        match self {
            Self::HttpHeader(http_header) => http_header.to_uppercase,
            Self::HttpQuery(http_query) => http_query.to_uppercase,
            Self::HttpBody(http_body) => http_body.to_uppercase,
            Self::HttpFormData(http_form_data) => http_form_data.to_uppercase,
            Self::HttpBodyRaw(http_body_raw) => http_body_raw.to_uppercase,
            Self::HttpPath(http_path) => http_path.to_uppercase,
        }
    }

    pub fn has_trim_attribute(&self) -> bool {
        let result = match self {
            Self::HttpHeader(http_header) => http_header.trim,
            Self::HttpQuery(http_query) => http_query.trim,
            Self::HttpBody(http_body) => http_body.trim,
            Self::HttpFormData(http_form_data) => http_form_data.trim,
            Self::HttpBodyRaw(http_body_raw) => http_body_raw.trim,
            Self::HttpPath(http_path) => http_path.trim,
        };

        result
    }
}

impl<'s> Into<HttpFieldAttribute<'s>> for HttpHeaderAttribute<'s> {
    fn into(self) -> HttpFieldAttribute<'s> {
        HttpFieldAttribute::HttpHeader(self)
    }
}

impl<'s> Into<HttpFieldAttribute<'s>> for HttpQueryAttribute<'s> {
    fn into(self) -> HttpFieldAttribute<'s> {
        HttpFieldAttribute::HttpQuery(self)
    }
}

impl<'s> Into<HttpFieldAttribute<'s>> for HttpBodyAttribute<'s> {
    fn into(self) -> HttpFieldAttribute<'s> {
        HttpFieldAttribute::HttpBody(self)
    }
}

impl<'s> Into<HttpFieldAttribute<'s>> for HttpFormDataAttribute<'s> {
    fn into(self) -> HttpFieldAttribute<'s> {
        HttpFieldAttribute::HttpFormData(self)
    }
}

impl<'s> Into<HttpFieldAttribute<'s>> for HttpBodyRawAttribute<'s> {
    fn into(self) -> HttpFieldAttribute<'s> {
        HttpFieldAttribute::HttpBodyRaw(self)
    }
}

impl<'s> Into<HttpFieldAttribute<'s>> for HttpPathAttribute<'s> {
    fn into(self) -> HttpFieldAttribute<'s> {
        HttpFieldAttribute::HttpPath(self)
    }
}
