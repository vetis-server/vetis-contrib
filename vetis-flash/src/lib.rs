use serde::{Deserialize, Serialize};
use vetis::{
    VetisResult,
    errors::{ConfigError, VetisError},
    host::path::{Path, PathConfig},
};

use crate::tokio::FlashPath;

#[cfg(feature = "runtime-tokio")]
/// Tokio runtime support module
pub mod tokio;

#[cfg(test)]
mod tests;

/// Builder for creating `FlashPathConfig` instances.
pub struct FlashPathConfigBuilder {
    uri: String,
    response: Option<String>,
    template: Option<String>,
    status_code: u16,
    headers: Option<Vec<(String, String)>>,
}

impl FlashPathConfigBuilder {
    /// Allow set the URI of static path.
    ///
    /// # Returns
    ///
    /// * `Self` - The builder.
    pub fn uri(mut self, uri: &str) -> Self {
        self.uri = uri.to_string();
        self
    }

    /// Allow set the response of flash path.
    ///
    /// # Returns
    ///
    /// * `Self` - The builder.
    pub fn response(mut self, extensions: &str) -> Self {
        self.response = Some(extensions.into());
        self
    }

    /// Allow set the template of flash path.
    ///
    /// # Returns
    ///
    /// * `Self` - The builder.
    pub fn template(mut self, template: &str) -> Self {
        self.template = Some(template.to_string());
        self
    }

    /// Allow set the status code of flash path.
    ///
    /// # Returns
    ///
    /// * `Self` - The builder.
    pub fn status_code(mut self, status_code: u16) -> Self {
        self.status_code = status_code;
        self
    }

    /// Allow set the headers of flash path.
    ///
    /// # Returns
    ///
    /// * `Self` - The builder.
    pub fn headers(mut self, heders: Vec<(String, String)>) -> Self {
        self.headers = Some(heders);
        self
    }

    /// Build the `StaticPathConfig` with the configured settings.
    ///
    /// # Returns
    ///
    /// * `VetisResult<FlashConfig>` - The `StaticPathConfig` with the configured settings.
    pub fn build(self) -> VetisResult<FlashPathConfig> {
        if self.uri.is_empty() {
            return Err(VetisError::Config(ConfigError::Path("URI cannot be empty".to_string())));
        }
        if self
            .response
            .is_none()
            && self
                .template
                .is_none()
        {
            return Err(VetisError::Config(ConfigError::Path(
                "You must provide response or template file".to_string(),
            )));
        }

        Ok(FlashPathConfig {
            uri: self.uri,
            response: self.response,
            template: self.template,
            status_code: self.status_code,
            headers: self.headers,
        })
    }
}

/// Configuration for static file serving.
#[derive(Clone, Serialize, Deserialize)]
pub struct FlashPathConfig {
    uri: String,
    response: Option<String>,
    template: Option<String>,
    status_code: u16,
    headers: Option<Vec<(String, String)>>,
}

#[typetag::serde(name = "flash")]
impl PathConfig for FlashPathConfig {
    fn uri(&mut self, uri: &str) {
        self.uri = uri.to_string();
    }

    fn boxed_clone(&self) -> Box<dyn PathConfig> {
        Box::new(self.clone())
    }

    fn boxed_path(&self) -> Box<dyn Path> {
        Box::new(FlashPath::new(self.clone()))
    }

    fn boxed_sync(&self) -> Box<dyn Path + Send + Sync> {
        Box::new(FlashPath::new(self.clone()))
    }
}

impl FlashPathConfig {
    /// Allow create a new `FlashPathConfigBuilder` with default settings.
    ///
    /// # Returns
    ///
    /// * `StaticPathConfigBuilder` - The builder.
    pub fn builder() -> FlashPathConfigBuilder {
        FlashPathConfigBuilder {
            uri: "/".to_string(),
            response: None,
            template: None,
            status_code: 200,
            headers: None,
        }
    }

    /// Returns uri
    ///
    /// # Returns
    ///
    /// * `&str` - The uri.
    pub fn uri(&self) -> &str {
        &self.uri
    }

    /// Returns response
    ///
    /// # Returns
    ///
    /// * `&str` - The response.
    pub fn response(&self) -> &Option<String> {
        &self.response
    }

    /// Returns template
    ///
    /// # Returns
    ///
    /// * `&str` - The template.
    pub fn template(&self) -> &Option<String> {
        &self.template
    }

    /// Returns status code
    ///
    /// # Returns
    ///
    /// * `&Option<Vec<String>>` - The index_files.
    pub fn status_code(&self) -> u16 {
        self.status_code
    }

    /// Returns headers
    ///
    /// # Returns
    ///
    /// * `&Option<Vec<String, String>>` - The headers.
    pub fn headers(&self) -> &Option<Vec<(String, String)>> {
        &self.headers
    }
}
