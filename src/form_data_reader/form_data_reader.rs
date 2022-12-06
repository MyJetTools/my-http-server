use crate::{FormDataItem, HttpFailResult};

use super::content_iterator::ContentIterator;

pub struct FormDataReader<'s> {
    data: Vec<FormDataItem<'s>>,
}

impl<'s> FormDataReader<'s> {
    pub fn new(content: &'s [u8]) -> Self {
        let mut data = Vec::new();

        for chunk in ContentIterator::new(content) {
            let item = FormDataItem::parse(chunk);
            data.push(item);
        }

        Self { data }
    }
    pub fn get_required(&'s self, name: &str) -> Result<&'s FormDataItem, HttpFailResult> {
        for itm in &self.data {
            if itm.get_name() == name {
                return Ok(itm);
            }
        }

        HttpFailResult::required_parameter_is_missing(name, "form data").into_err()
    }

    pub fn get_optional(&'s self, name: &str) -> Option<&'s FormDataItem> {
        for itm in &self.data {
            if itm.get_name() == name {
                return Some(itm);
            }
        }

        None
    }
}
