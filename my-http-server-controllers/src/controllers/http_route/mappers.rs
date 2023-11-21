use my_http_server_core::HttpFailResult;
use rust_extensions::date_time::DateTimeAsMicroseconds;

use crate::PathValue;

const SRC_PATH: &str = "Path";

impl TryInto<String> for PathValue<'_> {
    type Error = HttpFailResult;
    fn try_into(self) -> Result<String, Self::Error> {
        Ok(self.as_str().to_string())
    }
}

impl TryInto<DateTimeAsMicroseconds> for PathValue<'_> {
    type Error = HttpFailResult;
    fn try_into(self) -> Result<DateTimeAsMicroseconds, Self::Error> {
        my_http_server_core::convert_from_str::to_date_time(self.name, self.as_str(), SRC_PATH)
    }
}

impl TryInto<bool> for PathValue<'_> {
    type Error = HttpFailResult;
    fn try_into(self) -> Result<bool, Self::Error> {
        my_http_server_core::convert_from_str::to_bool(self.name, self.as_str(), SRC_PATH)
    }
}

impl TryInto<u8> for PathValue<'_> {
    type Error = HttpFailResult;
    fn try_into(self) -> Result<u8, Self::Error> {
        my_http_server_core::convert_from_str::to_simple_value(self.name, self.as_str(), SRC_PATH)
    }
}

impl TryInto<i8> for PathValue<'_> {
    type Error = HttpFailResult;
    fn try_into(self) -> Result<i8, Self::Error> {
        my_http_server_core::convert_from_str::to_simple_value(self.name, self.as_str(), SRC_PATH)
    }
}

impl TryInto<u16> for PathValue<'_> {
    type Error = HttpFailResult;
    fn try_into(self) -> Result<u16, Self::Error> {
        my_http_server_core::convert_from_str::to_simple_value(self.name, self.as_str(), SRC_PATH)
    }
}

impl TryInto<i16> for PathValue<'_> {
    type Error = HttpFailResult;
    fn try_into(self) -> Result<i16, Self::Error> {
        my_http_server_core::convert_from_str::to_simple_value(self.name, self.as_str(), SRC_PATH)
    }
}

impl TryInto<u32> for PathValue<'_> {
    type Error = HttpFailResult;
    fn try_into(self) -> Result<u32, Self::Error> {
        my_http_server_core::convert_from_str::to_simple_value(self.name, self.as_str(), SRC_PATH)
    }
}

impl TryInto<i32> for PathValue<'_> {
    type Error = HttpFailResult;
    fn try_into(self) -> Result<i32, Self::Error> {
        my_http_server_core::convert_from_str::to_simple_value(self.name, self.as_str(), SRC_PATH)
    }
}

impl TryInto<u64> for PathValue<'_> {
    type Error = HttpFailResult;
    fn try_into(self) -> Result<u64, Self::Error> {
        my_http_server_core::convert_from_str::to_simple_value(self.name, self.as_str(), SRC_PATH)
    }
}

impl TryInto<i64> for PathValue<'_> {
    type Error = HttpFailResult;
    fn try_into(self) -> Result<i64, Self::Error> {
        my_http_server_core::convert_from_str::to_simple_value(self.name, self.as_str(), SRC_PATH)
    }
}

impl TryInto<usize> for PathValue<'_> {
    type Error = HttpFailResult;
    fn try_into(self) -> Result<usize, Self::Error> {
        my_http_server_core::convert_from_str::to_simple_value(self.name, self.as_str(), SRC_PATH)
    }
}

impl TryInto<isize> for PathValue<'_> {
    type Error = HttpFailResult;
    fn try_into(self) -> Result<isize, Self::Error> {
        my_http_server_core::convert_from_str::to_simple_value(self.name, self.as_str(), SRC_PATH)
    }
}

impl TryInto<f32> for PathValue<'_> {
    type Error = HttpFailResult;
    fn try_into(self) -> Result<f32, Self::Error> {
        my_http_server_core::convert_from_str::to_simple_value(self.name, self.as_str(), SRC_PATH)
    }
}

impl TryInto<f64> for PathValue<'_> {
    type Error = HttpFailResult;
    fn try_into(self) -> Result<f64, Self::Error> {
        my_http_server_core::convert_from_str::to_simple_value(self.name, self.as_str(), SRC_PATH)
    }
}
