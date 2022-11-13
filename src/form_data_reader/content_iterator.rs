pub struct ContentIterator<'s> {
    boundary: &'s [u8],
    payload: &'s [u8],
    pos: usize,
}

impl<'s> ContentIterator<'s> {
    pub fn new(boundary: &'s [u8], payload: &'s [u8]) -> Self {
        Self {
            boundary,
            payload,
            pos: 0,
        }
    }
}

impl<'s> Iterator for ContentIterator<'s> {
    type Item = &'s [u8];

    fn next(&mut self) -> Option<Self::Item> {
        self.pos += self.boundary.len();
        self.pos = find_non_space(self.payload, self.pos)?;

        let next_pos = rust_extensions::slice_of_u8_utils::find_sequence_pos(
            self.payload,
            self.boundary,
            self.pos,
        )?;

        let result = &self.payload[self.pos..next_pos];

        self.pos = next_pos;
        Some(result)
    }
}

fn find_non_space(payload: &[u8], pos_from: usize) -> Option<usize> {
    for i in pos_from..payload.len() {
        let b = payload[i];
        if b > 32 {
            return Some(i);
        }
    }

    None
}

#[cfg(test)]
mod tests {

    use super::ContentIterator;

    #[test]
    fn test_splitting() {
        let boundary = "------WebKitFormBoundaryvrDBuVcszaZRkg3v";
        let payload: Vec<u8> = vec![
            45, 45, 45, 45, 45, 45, 87, 101, 98, 75, 105, 116, 70, 111, 114, 109, 66, 111, 117,
            110, 100, 97, 114, 121, 118, 114, 68, 66, 117, 86, 99, 115, 122, 97, 90, 82, 107, 103,
            51, 118, 13, 10, 67, 111, 110, 116, 101, 110, 116, 45, 68, 105, 115, 112, 111, 115,
            105, 116, 105, 111, 110, 58, 32, 102, 111, 114, 109, 45, 100, 97, 116, 97, 59, 32, 110,
            97, 109, 101, 61, 34, 100, 116, 70, 114, 111, 109, 34, 13, 10, 13, 10, 50, 13, 10, 45,
            45, 45, 45, 45, 45, 87, 101, 98, 75, 105, 116, 70, 111, 114, 109, 66, 111, 117, 110,
            100, 97, 114, 121, 118, 114, 68, 66, 117, 86, 99, 115, 122, 97, 90, 82, 107, 103, 51,
            118, 13, 10, 67, 111, 110, 116, 101, 110, 116, 45, 68, 105, 115, 112, 111, 115, 105,
            116, 105, 111, 110, 58, 32, 102, 111, 114, 109, 45, 100, 97, 116, 97, 59, 32, 110, 97,
            109, 101, 61, 34, 100, 116, 70, 114, 111, 109, 79, 112, 116, 34, 13, 10, 13, 10, 51,
            13, 10, 45, 45, 45, 45, 45, 45, 87, 101, 98, 75, 105, 116, 70, 111, 114, 109, 66, 111,
            117, 110, 100, 97, 114, 121, 118, 114, 68, 66, 117, 86, 99, 115, 122, 97, 90, 82, 107,
            103, 51, 118, 45, 45, 13, 10,
        ];

        for itm in ContentIterator::new(boundary.as_bytes(), payload.as_slice()) {
            let line = std::str::from_utf8(itm).unwrap();
            println!("{:?}", line);
            println!("----");
        }
    }
}
