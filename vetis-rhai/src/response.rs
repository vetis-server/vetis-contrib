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

pub fn add_header(response: &mut RhaiResponse, name: &str, value: &str) {
    response
        .headers
        .insert(name.into(), value.into());
}

pub fn write(response: &mut RhaiResponse, text: &str) {
    response
        .body
        .extend_from_slice(text.as_bytes());
}

impl RhaiResponse {
    pub fn new() -> Self {
        Self { status_code: 200, headers: Map::new(), body: BytesMut::new() }
    }

    pub fn set_status_code(&mut self, code: i64) {
        self.status_code = code;
    }

    pub fn get_status_code(&mut self) -> i64 {
        self.status_code
    }

    pub fn get_headers(&mut self) -> Map {
        self.headers.clone()
    }

    pub fn get_body(&mut self) -> &BytesMut {
        &self.body
    }
}
