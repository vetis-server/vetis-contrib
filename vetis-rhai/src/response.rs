#![allow(unused)]
use bytes::BytesMut;
use rhai::{Dynamic, Map};

#[derive(Clone)]
/// Internal rhai response
pub struct RhaiResponse {
    status_code: i64,
    headers: Map,
    body: BytesMut,
}

/// Add header to rhai response
pub fn add_header(response: &mut RhaiResponse, name: &str, value: &str) {
    response
        .headers
        .insert(name.into(), value.into());
}

/// Write text into response body
pub fn write(response: &mut RhaiResponse, text: &str) {
    response
        .body
        .extend_from_slice(text.as_bytes());
}

impl RhaiResponse {
    /// Create new RhaiResponse
    pub fn new() -> Self {
        Self { status_code: 200, headers: Map::new(), body: BytesMut::new() }
    }

    /// Allow set status code
    pub fn set_status_code(&mut self, code: i64) {
        self.status_code = code;
    }

    /// Returns status code
    pub fn get_status_code(&mut self) -> i64 {
        self.status_code
    }

    /// Returns response headers
    pub fn get_headers(&mut self) -> Map {
        self.headers.clone()
    }

    /// Returns response body
    pub fn get_body(&mut self) -> &BytesMut {
        &self.body
    }
}
