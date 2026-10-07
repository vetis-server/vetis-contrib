#![allow(unused)]
use bytes::BytesMut;
use rhai::{Blob, Dynamic, Map};
use rkyv::{Archive, Deserialize, Serialize};

#[derive(Clone, Deserialize, Serialize, Archive)]
/// Internal rhai response
pub struct Response {
    status_code: i64,
    headers: Vec<(String, String)>,
    body: Blob,
}

/// Add header to rhai response
pub fn add_header(response: &mut Response, name: &str, value: &str) {
    response
        .headers
        .push((name.into(), value.into()));
}

/// Write text into response body
pub fn write(response: &mut Response, text: &str) {
    response
        .body
        .extend_from_slice(text.as_bytes());
}

impl Response {
    /// Create new RhaiResponse
    pub fn new() -> Self {
        Self { status_code: 200, headers: Vec::new(), body: Blob::new() }
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
    pub fn get_headers(&mut self) -> Vec<(String, String)> {
        self.headers.clone()
    }

    /// Returns response headers
    pub fn set_headers(&mut self, headers: Vec<(String, String)>) {
        self.headers = headers
    }

    /// Returns response body
    pub fn get_body(&mut self) -> &Blob {
        &self.body
    }
}
