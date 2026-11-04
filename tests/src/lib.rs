#[cfg(test)]
pub mod test_http_input;

#[cfg(test)]
pub mod test_macros_split;

#[cfg(test)]
pub mod test_parse_e2e;

#[cfg(test)]
pub mod test_body_as_stream_e2e;

#[cfg(test)]
pub mod test_content_encoding_e2e;

#[cfg(test)]
pub mod test_body_from_stream_e2e;

#[cfg(test)]
pub mod test_pkg_compile_date_time;

#[cfg(test)]
pub mod test_static_files_e2e;

#[cfg(test)]
pub mod test_stream_compression_e2e;

#[cfg(test)]
pub mod test_protobuf_body_e2e;

#[cfg(test)]
pub mod test_if_none_match_e2e;

#[cfg(test)]
pub mod test_malformed_request_e2e;

#[cfg(test)]
pub mod test_connections_counter_e2e;

#[cfg(test)]
pub mod test_header_read_timeout_e2e;
