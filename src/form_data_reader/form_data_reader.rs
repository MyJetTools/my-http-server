use std::collections::HashMap;

use crate::{HttpFailResult, ValueAsString};

use super::content_iterator::ContentIterator;

pub struct Pos {
    start: usize,
    end: usize,
}

pub struct MultipartPiece {
    content_disposition: Pos,
    content_type: Option<Pos>,
}

pub struct FormDataReader<'s> {
    data: HashMap<String, MultipartPiece>,
    content: &'s [u8],
}

impl<'s> FormDataReader<'s> {
    pub fn new(boundary: &'s [u8], content: &'s [u8]) -> Self {
        let mut boundary_data = [b'-'; 255];
        boundary_data[2..2 + boundary.len()].copy_from_slice(boundary);

        let boundary = &boundary_data[..boundary.len() + 2];
        //Debugging
        println!("Boundary: {:?}", std::str::from_utf8(boundary));

        for chunk in ContentIterator::new(boundary, content) {
            println!("chunk: {:?}", chunk);
        }

        Self {
            data: HashMap::new(),
            content,
        }
    }
    pub fn get_required(&'s self, name: &'s str) -> Result<ValueAsString<'s>, HttpFailResult> {
        todo!("Not implemented yet")
    }

    pub fn get_optional(&'s self, name: &'s str) -> Option<ValueAsString<'s>> {
        todo!("Not implemented yet")
    }
}

/*
fn parse<'s>(boundary: &'s [u8], content: &'s [u8]) -> HashMap<String, &'s [u8]> {
    let mut result = HashMap::new();

    let mut file = 0;

    let mut content_type = None;

    let

    result
}

 */
