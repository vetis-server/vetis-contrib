#![allow(unused, dead_code)]
use rkyv::{Archive, Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize, Archive)]
#[rkyv(derive(Debug))]
/// Internal rhai response
pub struct Request {
    pub(crate) uri: String,
    pub(crate) script_path: String,
    pub(crate) method: String,
    pub(crate) version: String,
    pub(crate) headers: Vec<(String, String)>,
}

/// Add a header to request
pub fn add_header(request: &mut Request, name: &str, value: &str) {
    request
        .headers
        .push((name.into(), value.into()));
}

impl Request {
    /// Create a new Request instance
    pub fn new(uri: &str) -> Self {
        Self {
            uri: uri.into(),
            script_path: "index.rhai".into(),
            method: "GET".into(),
            version: "HTTP/1.1.".into(),
            headers: Vec::new(),
        }
    }

    /// Allot set script path
    pub fn set_script_path(&mut self, path: &str) {
        self.script_path = path.into()
    }

    /// Returns script path
    pub fn get_script_path(&mut self) -> &str {
        &self.script_path
    }

    /// Allow set request method
    pub fn set_method(&mut self, method: &str) {
        self.method = method.into();
    }

    /// Returns request method
    pub fn get_method(&mut self) -> &str {
        &self.method
    }

    /// Allow set http protocol version
    pub fn set_version(&mut self, version: &str) {
        self.version = version.into();
    }

    /// Returns http protocol version
    pub fn get_version(&mut self) -> &str {
        &self.version
    }

    /// Returns request headers
    pub fn get_headers(&mut self) -> &Vec<(String, String)> {
        &self.headers
    }
}
