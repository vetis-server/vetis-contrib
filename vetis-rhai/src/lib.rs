use crate::tokio::RhaiPath;
use serde::{Deserialize, Serialize};
use vetis::{
    VetisResult,
    errors::{ConfigError, VetisError},
    host::path::{Path, PathConfig},
};

pub mod request;
pub mod response;

#[cfg(test)]
mod tests;

#[cfg(feature = "runtime-tokio")]
/// Tokio runtime support module
pub mod tokio;

/// Builder for creating `RhaiPathConfig` instances.
pub struct RhaiPathConfigBuilder {
    uri: String,
    script: String,
}

/// TODO: Add link setter

impl RhaiPathConfigBuilder {
    /// Allow set the URI of static path.
    ///
    /// # Returns
    ///
    /// * `Self` - The builder.
    pub fn uri(mut self, uri: &str) -> Self {
        self.uri = uri.to_string();
        self
    }

    /// Allow set the script of rhai path.
    ///
    /// # Returns
    ///
    /// * `Self` - The builder.
    pub fn script(mut self, script: &str) -> Self {
        self.script = script.to_string();
        self
    }

    /// Build the `StaticPathConfig` with the configured settings.
    ///
    /// # Returns
    ///
    /// * `VetisResult<FlashConfig>` - The `StaticPathConfig` with the configured settings.
    pub fn build(self) -> VetisResult<RhaiPathConfig> {
        if self.uri.is_empty() {
            return Err(VetisError::Config(ConfigError::Path("URI cannot be empty".to_string())));
        }

        Ok(RhaiPathConfig { uri: self.uri, script: self.script })
    }
}

/// TODO: Add link getter

/// Configuration for static file serving.
#[derive(Clone, Serialize, Deserialize)]
pub struct RhaiPathConfig {
    uri: String,
    script: String,
}

#[typetag::serde(name = "rhai_path")]
impl PathConfig for RhaiPathConfig {
    fn uri(&mut self, uri: &str) {
        self.uri = uri.to_string();
    }

    fn boxed_clone(&self) -> Box<dyn PathConfig> {
        Box::new(self.clone())
    }

    fn boxed_path(&self) -> Box<dyn Path> {
        Box::new(RhaiPath::new(self.clone()))
    }

    fn boxed_sync(&self) -> Box<dyn Path + Send + Sync> {
        Box::new(RhaiPath::new(self.clone()))
    }
}

impl RhaiPathConfig {
    /// Allow create a new `RhaiPathConfigBuilder` with default settings.
    ///
    /// # Returns
    ///
    /// * `StaticPathConfigBuilder` - The builder.
    pub fn builder() -> RhaiPathConfigBuilder {
        RhaiPathConfigBuilder { uri: "/".to_string(), script: "index.rhai".into() }
    }

    /// Returns uri
    ///
    /// # Returns
    ///
    /// * `&str` - The uri.
    pub fn uri(&self) -> &str {
        &self.uri
    }

    /// Returns script
    ///
    /// # Returns
    ///
    /// * `&str` - The script.
    pub fn script(&self) -> &str {
        &self.script
    }
}
