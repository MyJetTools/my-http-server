use my_http_server_core::{HttpFailResult, HttpPath, InputParamValue};

use super::HttpRouteSegment;

pub struct HttpRoute {
    pub route: String,
    pub keys_amount: usize,
    segments: Vec<HttpRouteSegment>,
}

impl HttpRoute {
    pub fn new(route: &str) -> Self {
        let route_as_bytes = route.as_bytes();

        let mut keys_amount = 0;

        let mut segments = Vec::new();
        let mut prev_pos = None;

        let mut last_element: u8 = 0;

        for pos in 0..route_as_bytes.len() {
            last_element = route_as_bytes[pos];
            if last_element == b'/' {
                if let Some(prev_pos) = prev_pos {
                    let segment = &route_as_bytes[prev_pos + 1..pos];
                    let segment = HttpRouteSegment::new(segment);

                    if segment.is_key() {
                        keys_amount += 1;
                    }

                    segments.push(segment);
                }

                prev_pos = Some(pos);
            }
        }

        if last_element != b'/' {
            if let Some(prev_pos) = prev_pos {
                let segment = &route_as_bytes[prev_pos + 1..];
                let segment = HttpRouteSegment::new(segment);

                if segment.is_key() {
                    keys_amount += 1;
                }

                segments.push(segment);
            }
        }

        Self {
            keys_amount,
            segments,
            route: route.to_string(),
        }
    }

    pub fn is_my_path(&self, path: &HttpPath) -> bool {
        if path.segments_amount() != self.segments.len() {
            return false;
        }

        if path.is_root() && self.segments.len() == 0 {
            return true;
        }

        let mut index = 0;
        for segment in &self.segments {
            match segment {
                HttpRouteSegment::Key(_) => {}
                HttpRouteSegment::Segment(value) => {
                    if !path.has_value_at_index_case_insensitive(index, value) {
                        return false;
                    }
                }
            }

            index += 1;
        }

        true
    }

    pub fn get_value<'s>(
        &'s self,
        path: &'s HttpPath,
        key: &str,
    ) -> Result<InputParamValue<'s>, HttpFailResult> {
        if self.keys_amount == 0 {
            return Err(HttpFailResult::new(
                my_http_server_core::WebContentType::Text,
                400,
                format!("Route {} does not contain any keys", self.route).into_bytes(),
                false,
                true,
            ));
        }

        let mut index = 0;
        for segment in &self.segments {
            match segment {
                HttpRouteSegment::Key(segment_key) => {
                    if segment_key == key {
                        match path.get_segment_value_as_str(index) {
                            Some(value) => return Ok(InputParamValue::Raw { value, src: "path" }),
                            None => {
                                panic!("Should not be here");
                            }
                        }
                    }
                }
                HttpRouteSegment::Segment(_) => {}
            }

            index += 1;
        }

        return Err(HttpFailResult::new(
            my_http_server_core::WebContentType::Text,
            400,
            format!("Route {} does not have key {}", self.route, key).into_bytes(),
            false,
            true,
        ));
    }

    pub fn has_route_key(&self, key: &str) -> bool {
        for segment in &self.segments {
            match segment {
                HttpRouteSegment::Key(segment_key) => {
                    if segment_key == key {
                        return true;
                    }
                }
                HttpRouteSegment::Segment(_) => {}
            }
        }

        false
    }

    pub fn check_route_keys(&self, route_keys: &Vec<&'static str>) -> Result<(), String> {
        for key in route_keys {
            if !self.has_route_key(key) {
                return Err(format!(
                    "Route '{}' does not have key '{}'",
                    self.route, key
                ));
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parsing_segments() {
        let route = HttpRoute::new("/test/{key}/second");

        assert_eq!(route.segments.len(), 3);
        assert_eq!(route.segments[0].unwrap_as_segment(), "test");
        assert_eq!(route.segments[1].unwrap_as_key(), "key");
        assert_eq!(route.segments[2].unwrap_as_segment(), "second");
    }

    #[test]
    fn test_parsing_segments_with_last_slash() {
        let route = HttpRoute::new("/test/{key}/second/");

        assert_eq!(route.segments.len(), 3);
        assert_eq!(route.segments[0].unwrap_as_segment(), "test");
        assert_eq!(route.segments[1].unwrap_as_key(), "key");
        assert_eq!(route.segments[2].unwrap_as_segment(), "second");
    }

    #[test]
    fn general_test() {
        let route = HttpRoute::new("/test/{key}/second");
        let path = HttpPath::from_str("/test/1/second");
        assert_eq!(route.is_my_path(&path), true);

        assert_eq!(
            route
                .get_value(&path, "key")
                .unwrap()
                .get_raw_str()
                .unwrap(),
            "1"
        );
    }

    #[test]
    fn test_root() {
        let route = HttpRoute::new("/");
        let path = HttpPath::from_str("/");
        assert_eq!(route.is_my_path(&path), true);
    }

    #[test]
    fn test_not_my_route() {
        let route = HttpRoute::new("/test/{key}/second");
        let path = HttpPath::from_str("/test/1/");
        assert_eq!(route.is_my_path(&path), false);

        let path = HttpPath::from_str("/test/1");
        assert_eq!(route.is_my_path(&path), false);
    }

    #[test]
    fn test_not_my_route_2() {
        let route = HttpRoute::new("/test/{key}/second");
        let path = HttpPath::from_str("/test/1/second/4/");
        assert_eq!(route.is_my_path(&path), false);

        let path = HttpPath::from_str("/test/1/second/4");
        assert_eq!(route.is_my_path(&path), false);
    }

    #[test]
    fn check_has_route_key() {
        let route = HttpRoute::new("/test/{key}/second");
        assert_eq!(route.has_route_key("key"), true);
        assert_eq!(route.has_route_key("Key"), false);
        assert_eq!(route.has_route_key("key2"), false);

        let route = HttpRoute::new("/test/{Key}/second");
        assert_eq!(route.has_route_key("Key"), true);
        assert_eq!(route.has_route_key("key"), false);
    }
}
