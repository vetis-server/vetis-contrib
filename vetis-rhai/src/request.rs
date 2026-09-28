#![allow(unused, dead_code)]
#[derive(Clone)]
/// Internal rhai response
pub struct RhaiRequest {
    uri: String,
    script_path: String,
    method: String,
    version: String,
    headers: Vec<(String, String)>,
}

pub fn add_header(request: &mut RhaiRequest, name: &str, value: &str) {
    request
        .headers
        .push((name.into(), value.into()));
}

impl RhaiRequest {
    pub fn new(uri: &str) -> Self {
        Self {
            uri: uri.into(),
            script_path: "index.rhai".into(),
            method: "GET".into(),
            version: "HTTP/1.1.".into(),
            headers: Vec::new(),
        }
    }

    pub fn get_script_path(&mut self) -> &str {
        &self.script_path
    }

    pub fn set_method(&mut self, method: &str) {
        self.method = method.into();
    }

    pub fn get_method(&mut self) -> &str {
        &self.method
    }

    pub fn set_version(&mut self, version: &str) {
        self.version = version.into();
    }

    pub fn get_version(&mut self) -> &str {
        &self.version
    }

    pub fn get_headers(&mut self) -> &Vec<(String, String)> {
        &self.headers
    }
}
