pub struct ContentIterator<'s> {
    boundary_data: [u8; 255],
    boundary_len: usize,
    payload: &'s [u8],
    pos: usize,
}

impl<'s> ContentIterator<'s> {
    pub fn new(boundary: &[u8], payload: &'s [u8]) -> Self {
        let mut result = Self {
            boundary_data: [b'-'; 255],
            boundary_len: boundary.len() + 2,
            payload,
            pos: 0,
        };

        result.boundary_data[2..2 + boundary.len()].copy_from_slice(boundary);

        result
    }

    pub fn get_boundary(&'s self) -> &'s [u8] {
        &self.boundary_data[..self.boundary_len]
    }
}

impl<'s> Iterator for ContentIterator<'s> {
    type Item = &'s [u8];

    fn next(&mut self) -> Option<Self::Item> {
        self.pos += self.boundary_len;
        self.pos = find_non_space(self.payload, self.pos)?;

        let next_pos = rust_extensions::slice_of_u8_utils::find_sequence_pos(
            self.payload,
            self.get_boundary(),
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
        let boundary = "----WebKitFormBoundaryu7oxE5T3UC2xY2Q9";
        let payload: Vec<u8> = vec![
            45, 45, 45, 45, 45, 45, 87, 101, 98, 75, 105, 116, 70, 111, 114, 109, 66, 111, 117,
            110, 100, 97, 114, 121, 117, 55, 111, 120, 69, 53, 84, 51, 85, 67, 50, 120, 89, 50, 81,
            57, 13, 10, //End of boundary
            //Message-0
            67, 111, 110, 116, 101, 110, 116, 45, 68, 105, 115, 112, 111, 115, 105, 116, 105, 111,
            110, 58, 32, 102, 111, 114, 109, 45, 100, 97, 116, 97, 59, 32, 110, 97, 109, 101, 61,
            34, 100, 116, 70, 114, 111, 109, 34, 13, 10, 13, 10, 50, 13, 10,
            //Start of boundary
            45, 45, 45, 45, 45, 45, 87, 101, 98, 75, 105, 116, 70, 111, 114, 109, 66, 111, 117, 110,
            100, 97, 114, 121, 117, 55, 111, 120, 69, 53, 84, 51, 85, 67, 50, 120, 89, 50, 81, 57,
            13, 10, //End of boundary
            //Message-1
            67, 111, 110, 116, 101, 110, 116, 45, 68, 105, 115, 112, 111, 115, 105, 116, 105, 111,
            110, 58, 32, 102, 111, 114, 109, 45, 100, 97, 116, 97, 59, 32, 110, 97, 109, 101, 61,
            34, 100, 116, 70, 114, 111, 109, 79, 112, 116, 34, 13, 10, 13, 10, 51, 13, 10,
            //Start of boundary
            45, 45, 45, 45, 45, 45, 87, 101, 98, 75, 105, 116, 70, 111, 114, 109, 66, 111, 117, 110,
            100, 97, 114, 121, 117, 55, 111, 120, 69, 53, 84, 51, 85, 67, 50, 120, 89, 50, 81, 57,
            13, 10, //Message-2
            67, 111, 110, 116, 101, 110, 116, 45, 68, 105, 115, 112, 111, 115, 105, 116, 105, 111,
            110, 58, 32, 102, 111, 114, 109, 45, 100, 97, 116, 97, 59, 32, 110, 97, 109, 101, 61,
            34, 102, 105, 108, 101, 34, 59, 32, 102, 105, 108, 101, 110, 97, 109, 101, 61, 34, 116,
            101, 115, 116, 45, 112, 97, 121, 108, 111, 97, 100, 46, 116, 120, 116, 34, 13, 10, 67,
            111, 110, 116, 101, 110, 116, 45, 84, 121, 112, 101, 58, 32, 116, 101, 120, 116, 47,
            112, 108, 97, 105, 110, 13, 10, 13, 10, 49, 50, 51, 13, 10,
            //Start of boundary
            45, 45, 45, 45, 45, 45, 87, 101, 98, 75, 105, 116, 70, 111, 114, 109, 66, 111, 117, 110,
            100, 97, 114, 121, 117, 55, 111, 120, 69, 53, 84, 51, 85, 67, 50, 120, 89, 50, 81, 57,
            45, 45, 13, 10,
        ];

        let result: Vec<&[u8]> =
            ContentIterator::new(boundary.as_bytes(), payload.as_slice()).collect();

        let expected_payload_0: Vec<u8> = vec![
            67, 111, 110, 116, 101, 110, 116, 45, 68, 105, 115, 112, 111, 115, 105, 116, 105, 111,
            110, 58, 32, 102, 111, 114, 109, 45, 100, 97, 116, 97, 59, 32, 110, 97, 109, 101, 61,
            34, 100, 116, 70, 114, 111, 109, 34, 13, 10, 13, 10, 50, 13, 10,
        ];

        let expected_payload_1: Vec<u8> = vec![
            67, 111, 110, 116, 101, 110, 116, 45, 68, 105, 115, 112, 111, 115, 105, 116, 105, 111,
            110, 58, 32, 102, 111, 114, 109, 45, 100, 97, 116, 97, 59, 32, 110, 97, 109, 101, 61,
            34, 100, 116, 70, 114, 111, 109, 79, 112, 116, 34, 13, 10, 13, 10, 51, 13, 10,
        ];

        let expected_payload_2: Vec<u8> = vec![
            67, 111, 110, 116, 101, 110, 116, 45, 68, 105, 115, 112, 111, 115, 105, 116, 105, 111,
            110, 58, 32, 102, 111, 114, 109, 45, 100, 97, 116, 97, 59, 32, 110, 97, 109, 101, 61,
            34, 102, 105, 108, 101, 34, 59, 32, 102, 105, 108, 101, 110, 97, 109, 101, 61, 34, 116,
            101, 115, 116, 45, 112, 97, 121, 108, 111, 97, 100, 46, 116, 120, 116, 34, 13, 10, 67,
            111, 110, 116, 101, 110, 116, 45, 84, 121, 112, 101, 58, 32, 116, 101, 120, 116, 47,
            112, 108, 97, 105, 110, 13, 10, 13, 10, 49, 50, 51, 13, 10,
        ];

        assert_eq!(result.get(0).unwrap(), &expected_payload_0);
        assert_eq!(result.get(1).unwrap(), &expected_payload_1);
        assert_eq!(result.get(2).unwrap(), &expected_payload_2);
    }
}
