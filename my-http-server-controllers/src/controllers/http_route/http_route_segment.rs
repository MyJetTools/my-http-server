pub enum HttpRouteSegment {
    Key(String),
    Segment(String),
}

impl HttpRouteSegment {
    pub fn new(value: &[u8]) -> Self {
        if value.len() < 2 {
            return Self::Segment(std::str::from_utf8(value).unwrap().to_lowercase());
        }

        if value[0] == b'{' && value[value.len() - 1] == b'}' {
            return Self::Key(
                std::str::from_utf8(&value[1..value.len() - 1])
                    .unwrap()
                    .to_string(),
            );
        }

        return Self::Segment(std::str::from_utf8(value).unwrap().to_lowercase());
    }

    pub fn is_key(&self) -> bool {
        match self {
            HttpRouteSegment::Key(_) => true,
            _ => false,
        }
    }

    pub fn unwrap_as_key(&self) -> &str {
        match self {
            HttpRouteSegment::Key(value) => value,
            _ => panic!("This segment is not a key"),
        }
    }

    pub fn unwrap_as_segment(&self) -> &str {
        match self {
            HttpRouteSegment::Segment(value) => value,
            _ => panic!("This segment is not a segment"),
        }
    }
}
