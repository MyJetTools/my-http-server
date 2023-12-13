use rust_extensions::lazy::LazyVec;
use types_reader::{MacrosAttribute, StructProperty};

use super::InputField;
use crate::attributes::*;

pub struct HttpInputProperties<'s> {
    pub header_fields: Option<Vec<InputField<'s>>>,
    pub query_string_fields: Option<Vec<InputField<'s>>>,
    pub body_fields: Option<Vec<InputField<'s>>>,
    pub form_data_fields: Option<Vec<InputField<'s>>>,
    pub body_raw_field: Option<InputField<'s>>,
    pub path_fields: Option<Vec<InputField<'s>>>,

    pub print_request_to_console: bool,
}

impl<'s> HttpInputProperties<'s> {
    pub fn new(props: &'s [StructProperty]) -> Result<Self, syn::Error> {
        let mut print_request_to_console = false;
        let mut body_fields = LazyVec::with_capacity(props.len());
        let mut query_string_fields = LazyVec::with_capacity(props.len());
        let mut header_fields = LazyVec::with_capacity(props.len());

        let mut path_fields = LazyVec::with_capacity(props.len());

        let mut form_data_fields = LazyVec::with_capacity(props.len());

        let mut body_raw_field = None;

        for struct_property in props {
            if struct_property.attrs.has_attr(IgnoreAttribute::NAME) {
                continue;
            }

            let attr: Option<HttpQueryAttribute> = struct_property.try_get_attribute()?;

            if let Some(attr) = attr {
                if attr.print_request_to_console {
                    print_request_to_console = true;
                }
                query_string_fields.add(InputField::new(struct_property, attr));
                continue;
            }

            let attr: Option<HttpPathAttribute> = struct_property.try_get_attribute()?;

            if let Some(attr) = attr {
                if attr.print_request_to_console {
                    print_request_to_console = true;
                }
                path_fields.add(InputField::new(struct_property, attr));
                continue;
            }

            let attr: Option<HttpHeaderAttribute> = struct_property.try_get_attribute()?;

            if let Some(attr) = attr {
                if attr.print_request_to_console {
                    print_request_to_console = true;
                }
                header_fields.add(InputField::new(struct_property, attr));
                continue;
            }

            let attr: Option<HttpFormDataAttribute> = struct_property.try_get_attribute()?;

            if let Some(attr) = attr {
                if !body_fields.is_empty() {
                    struct_property.throw_error("http_body attribute already exists.")?;
                }

                if body_raw_field.is_some() {
                    struct_property.throw_error("body_raw attribute already exists.")?;
                }

                if attr.print_request_to_console {
                    print_request_to_console = true;
                }

                form_data_fields.add(InputField::new(struct_property, attr));
                continue;
            }

            let attr: Option<HttpBodyAttribute> = struct_property.try_get_attribute()?;

            if let Some(attr) = attr {
                if !form_data_fields.is_empty() {
                    struct_property.throw_error("form_data attribute already exists.")?;
                }

                if body_raw_field.is_some() {
                    struct_property.throw_error("body_raw attribute already exists.")?;
                }

                if attr.print_request_to_console {
                    print_request_to_console = true;
                }
                body_fields.add(InputField::new(struct_property, attr));
                continue;
            }

            let attr: Option<HttpBodyRawAttribute> = struct_property.try_get_attribute()?;

            if let Some(attr) = attr {
                if !form_data_fields.is_empty() {
                    struct_property.throw_error("form_data attribute already exists.")?;
                }

                if body_raw_field.is_some() {
                    struct_property.throw_error("body_raw attribute can be used once.")?;
                }

                if !body_fields.is_empty() {
                    struct_property.throw_error("http_body attribute already exists.")?;
                }

                if attr.print_request_to_console {
                    print_request_to_console = true;
                }

                body_raw_field = Some(InputField::new(struct_property, attr));

                continue;
            }
        }

        let result = Self {
            body_fields: body_fields.get_result(),
            header_fields: header_fields.get_result(),
            query_string_fields: query_string_fields.get_result(),
            path_fields: path_fields.get_result(),
            body_raw_field,
            form_data_fields: form_data_fields.get_result(),
            print_request_to_console,
        };

        result.self_check()?;

        Ok(result)
    }

    fn self_check(&self) -> Result<(), syn::Error> {
        if let Some(body_raw) = &self.body_raw_field {
            if self.body_fields.is_some() {
                let err = syn::Error::new_spanned(
                    body_raw.property.field,
                    "http_body data and http_body_raw can not be mixed",
                );
                return Err(err);
            }

            if self.form_data_fields.is_some() {
                let err = syn::Error::new_spanned(
                    body_raw.property.field,
                    "http_form_data and http_body_raw data can not be mixed",
                );
                return Err(err);
            }
        }

        if let Some(body_fields) = &self.body_fields {
            check_duplicated(body_fields)?;
            if let Some(body_raw) = &self.body_raw_field {
                let err = syn::Error::new_spanned(
                    body_raw.property.field,
                    "http_body_raw and http_body can not be mixed",
                );
                return Err(err);
            }

            if self.form_data_fields.is_some() {
                let err = syn::Error::new_spanned(
                    body_fields.get(0).unwrap().property.field,
                    "http_form_data and http_body data can not be mixed",
                );
                return Err(err);
            }
        }
        if let Some(form_data_fields) = &self.form_data_fields {
            check_duplicated(form_data_fields)?;
            if let Some(body_raw) = &self.body_raw_field {
                let err = syn::Error::new_spanned(
                    body_raw.property.field,
                    "http_body_raw and http_form_data data can not be mixed",
                );
                return Err(err);
            }

            if self.body_fields.is_some() {
                let err = syn::Error::new_spanned(
                    form_data_fields.get(0).unwrap().property.field,
                    "http_body and http_form_data can not be mixed",
                );
                return Err(err);
            }
        }

        if let Some(header_fields) = &self.header_fields {
            check_duplicated(header_fields)?;
        }

        if let Some(query_string_fields) = &self.query_string_fields {
            check_duplicated(query_string_fields)?;
        }

        if let Some(path_fields) = &self.path_fields {
            for path_field in path_fields {
                if path_field.property.ty.is_option() {
                    let err = syn::Error::new_spanned(
                        path_field.property.field,
                        "Path field can not be optional",
                    );
                    return Err(err);
                }
            }
        }

        Ok(())
    }

    pub fn get_all(&'s self) -> Vec<&'s InputField<'s>> {
        let mut result = Vec::new();

        if let Some(header_fields) = &self.header_fields {
            result.extend(header_fields);
        }

        if let Some(query_string_fields) = &self.query_string_fields {
            result.extend(query_string_fields);
        }

        if let Some(body_fields) = &self.body_fields {
            result.extend(body_fields);
        }

        if let Some(form_data_fields) = &self.form_data_fields {
            result.extend(form_data_fields);
        }

        if let Some(body_raw_field) = &self.body_raw_field {
            result.push(body_raw_field);
        }

        if let Some(path_fields) = &self.path_fields {
            result.extend(path_fields);
        }

        result
    }
}

fn check_duplicated(items: &[InputField]) -> Result<(), syn::Error> {
    for i in 0..items.len() {
        for j in 0..items.len() {
            if i == j {
                continue;
            }

            let one = items.get(i).unwrap();
            let another = items.get(j).unwrap();

            if one.get_input_field_name()? == another.get_input_field_name()? {
                if j > i {
                    return another.throw_error("Duplicated field name")?;
                } else {
                    return one.throw_error("Duplicated field name")?;
                }
            }
        }
    }
    Ok(())
}
