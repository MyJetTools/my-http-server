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
            self.boundary,
            self.payload,
            self.pos,
        )?;

        let result = &self.payload[self.pos..next_pos];

        self.pos = next_pos;
        Some(result)
    }
}

fn find_non_space(payload: &[u8], pos_from: usize) -> Option<usize> {
    for i in pos_from..payload.len() {
        if payload[i] > 32 {
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
        let boundary = b"------WebKitFormBoundaryXayIfSQWkEtJ6k10";
        let payload = r#"------WebKitFormBoundaryXayIfSQWkEtJ6k10
Content-Disposition: form-data; name="dtFrom"

2
------WebKitFormBoundaryXayIfSQWkEtJ6k10
Content-Disposition: form-data; name="dtFromOpt"
        
3
------WebKitFormBoundaryXayIfSQWkEtJ6k10"#;

        for itm in ContentIterator::new(boundary, payload.as_bytes()) {
            let line = std::str::from_utf8(itm).unwrap();
            println!("{:?}", line);
            println!("----");
        }
    }
}
