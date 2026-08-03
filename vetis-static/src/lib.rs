#![doc = include_str!("../README.md")]
#![deny(missing_docs)]
use serde::{Deserialize, Deserializer, Serialize};
#[cfg(unix)]
use std::{
    path::PathBuf,
    time::{Duration, SystemTime},
};
use time::{format_description::well_known::Rfc2822, OffsetDateTime};
use vetis::{
    errors::{ConfigError, VetisError},
    virtual_host::path::PathConfig,
    VetisResult,
};

//pub(crate) type VetisFileCache = Cache<String, StaticFile>;

const MAX_FILE_SIZE: usize = 10 * 1024 * 1024; // 10MB
const DEFAULT_TTL: Duration = Duration::from_secs(60);
const DEFAULT_TTI: Duration = Duration::from_secs(10);
const DEFAULT_CAPACITY: u64 = 1000;

/*
#[cfg(feature = "runtime-compio")]
/// Compio runtime support module
pub mod compio;
*/
#[cfg(feature = "runtime-smol")]
/// Smol runtime support module
pub mod smol;
#[cfg(feature = "runtime-tokio")]
/// Tokio runtime support module
pub mod tokio;

#[cfg(test)]
mod tests;

/// Format a SystemTime to a string in RFC 2822 format.
///
/// # Arguments
///
/// * `date` - The SystemTime to format.
///
/// # Returns
///
/// * `String` - The formatted date.
pub fn format_date(date: SystemTime) -> String {
    let date = OffsetDateTime::from(date);
    date.format(&Rfc2822)
        .unwrap()
}

/// Builder for creating `StaticPathCache` instances.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct StaticPathCacheBuilder {
    uri: String,
    max_file_size: usize,
    ttl: Duration,
    tti: Duration,
    capacity: u64,
}

#[typetag::serde]
impl PathConfig for StaticPathCacheBuilder {
    fn uri(&mut self, value: &str) {
        self.uri = value.to_string();
    }
}

impl StaticPathCacheBuilder {
    /// Set max file size
    pub fn max_file_size(mut self, max_file_size: usize) -> Self {
        self.max_file_size = max_file_size;
        self
    }

    /// Set time to live
    pub fn ttl(mut self, ttl: Duration) -> Self {
        self.ttl = ttl;
        self
    }

    /// Set time to idle
    pub fn tti(mut self, tti: Duration) -> Self {
        self.tti = tti;
        self
    }

    /// Set capacity
    pub fn capacity(mut self, capacity: u64) -> Self {
        self.capacity = capacity;
        self
    }

    /// Build the `StaticPathCache`
    pub fn build(self) -> StaticPathCache {
        StaticPathCache {
            max_file_size: self.max_file_size,
            ttl: self.ttl,
            tti: self.tti,
            capacity: self.capacity,
        }
    }
}

/// Configuration for static file caching.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StaticPathCache {
    max_file_size: usize,
    #[serde(deserialize_with = "deserialize_duration")]
    ttl: Duration,
    #[serde(deserialize_with = "deserialize_duration")]
    tti: Duration,
    capacity: u64,
}

impl Default for StaticPathCache {
    fn default() -> Self {
        Self {
            max_file_size: MAX_FILE_SIZE,
            ttl: DEFAULT_TTL,
            tti: DEFAULT_TTI,
            capacity: DEFAULT_CAPACITY,
        }
    }
}

/// Configuration for static file caching.
impl StaticPathCache {
    /// Create a new builder for `StaticPathCache`.
    pub fn builder() -> StaticPathCacheBuilder {
        StaticPathCacheBuilder {
            uri: String::new(),
            max_file_size: MAX_FILE_SIZE,
            ttl: DEFAULT_TTL,
            tti: DEFAULT_TTI,
            capacity: DEFAULT_CAPACITY,
        }
    }

    /// Return max file size
    pub fn max_file_size(&self) -> usize {
        self.max_file_size
    }

    /// Return time to live
    pub fn ttl(&self) -> Duration {
        self.ttl
    }

    /// Return time to idle
    pub fn tti(&self) -> Duration {
        self.tti
    }

    /// Return capacity
    pub fn capacity(&self) -> u64 {
        self.capacity
    }
}

/// Builder for creating `StaticPathConfig` instances.
pub struct StaticPathConfigBuilder {
    uri: String,
    extensions: String,
    directory: String,
    index_files: Option<Vec<String>>,
    cache: Option<StaticPathCache>,
}

impl StaticPathConfigBuilder {
    /// Allow set the URI of the static path.
    ///
    /// # Returns
    ///
    /// * `Self` - The builder.
    pub fn uri(mut self, uri: &str) -> Self {
        self.uri = uri.to_string();
        self
    }

    /// Allow set the extensions of the static path.
    ///
    /// # Returns
    ///
    /// * `Self` - The builder.
    pub fn extensions(mut self, extensions: &str) -> Self {
        self.extensions = extensions.to_string();
        self
    }

    /// Allow set the directory of the static path.
    ///
    /// # Returns
    ///
    /// * `Self` - The builder.
    pub fn directory(mut self, directory: &str) -> Self {
        self.directory = directory.to_string();
        self
    }

    /// Allow set the index files of the static path.
    ///
    /// # Returns
    ///
    /// * `Self` - The builder.
    pub fn index_files(mut self, index_files: Vec<String>) -> Self {
        self.index_files = Some(index_files);
        self
    }

    /// Allow set the cache of the static path.
    ///
    /// # Returns
    ///
    /// * `Self` - The builder.
    pub fn cache(mut self, cache: StaticPathCache) -> Self {
        self.cache = Some(cache);
        self
    }

    /// Build the `StaticPathConfig` with the configured settings.
    ///
    /// # Returns
    ///
    /// * `VetisResult<StaticPathConfig>` - The `StaticPathConfig` with the configured settings.
    pub fn build(self) -> VetisResult<StaticPathConfig> {
        if self.uri.is_empty() {
            return Err(VetisError::Config(ConfigError::Path("URI cannot be empty".to_string())));
        }
        if self
            .extensions
            .is_empty()
        {
            return Err(VetisError::Config(ConfigError::Path(
                "Extensions cannot be empty".to_string(),
            )));
        }
        if self
            .directory
            .is_empty()
        {
            return Err(VetisError::Config(ConfigError::Path(
                "Directory cannot be empty".to_string(),
            )));
        }

        Ok(StaticPathConfig {
            uri: self.uri,
            extensions: self.extensions,
            directory: self.directory,
            index_files: self.index_files,
            cache: self.cache,
        })
    }
}

/// Configuration for static file serving.
#[derive(Clone, Serialize, Deserialize)]
pub struct StaticPathConfig {
    uri: String,
    extensions: String,
    directory: String,
    index_files: Option<Vec<String>>,
    cache: Option<StaticPathCache>,
}

#[typetag::serde]
impl PathConfig for StaticPathConfig {
    fn uri(&mut self, uri: &str) {
        self.uri = uri.to_string();
    }
}

impl StaticPathConfig {
    /// Allow create a new `StaticPathConfigBuilder` with default settings.
    ///
    /// # Returns
    ///
    /// * `StaticPathConfigBuilder` - The builder.
    pub fn builder() -> StaticPathConfigBuilder {
        StaticPathConfigBuilder {
            uri: "/".to_string(),
            extensions: ".html".to_string(),
            directory: ".".to_string(),
            index_files: None,
            cache: Some(StaticPathCache::default()),
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

    /// Returns extensions
    ///
    /// # Returns
    ///
    /// * `&str` - The extensions.
    pub fn extensions(&self) -> &str {
        &self.extensions
    }

    /// Returns directory
    ///
    /// # Returns
    ///
    /// * `&str` - The directory.
    pub fn directory(&self) -> &str {
        &self.directory
    }

    /// Returns index_files
    ///
    /// # Returns
    ///
    /// * `&Option<Vec<String>>` - The index_files.
    pub fn index_files(&self) -> &Option<Vec<String>> {
        &self.index_files
    }

    /// Returns cache
    ///
    /// # Returns
    ///
    /// * `&Option<StaticPathCache>` - The cache.
    pub fn cache(&self) -> &Option<StaticPathCache> {
        &self.cache
    }
}

fn deserialize_duration<'de, D>(deserializer: D) -> Result<Duration, D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    duration_str::parse(&s).map_err(serde::de::Error::custom)
}

/// Builder for creating `StaticFileMetadata` instances.
#[derive(Debug, Clone)]
pub struct StaticFileMetadataBuilder {
    mime: Option<String>,
    size: u64,
    modified: std::time::SystemTime,
    etag: Option<String>,
}

impl StaticFileMetadataBuilder {
    /// Sets the mime type.
    ///
    /// # Arguments
    ///
    /// * `mime` - The mime type.
    ///
    /// # Returns
    ///
    /// * `Self` - The builder.
    pub fn mime(mut self, mime: String) -> Self {
        self.mime = Some(mime);
        self
    }

    /// Sets the size.
    ///
    /// # Arguments
    ///
    /// * `size` - The size.
    ///
    /// # Returns
    ///
    /// * `Self` - The builder.
    pub fn size(mut self, size: u64) -> Self {
        self.size = size;
        self
    }

    /// Sets the modified time.
    ///
    /// # Arguments
    ///
    /// * `modified` - The modified time.
    ///
    /// # Returns
    ///
    /// * `Self` - The builder.
    pub fn modified(mut self, modified: std::time::SystemTime) -> Self {
        self.modified = modified;
        self
    }

    /// Sets the etag.
    ///
    /// # Arguments
    ///
    /// * `etag` - The etag.
    ///
    /// # Returns
    ///
    /// * `Self` - The builder.
    pub fn etag(mut self, etag: String) -> Self {
        self.etag = Some(etag);
        self
    }

    /// Builds the `StaticFileMetadata`.
    ///
    /// # Returns
    ///
    /// * `StaticFileMetadata` - The built metadata.
    pub fn build(self) -> StaticFileMetadata {
        StaticFileMetadata {
            mime: self.mime,
            size: self.size,
            modified: self.modified,
            etag: self.etag,
        }
    }
}

#[derive(Clone, Debug)]
/// Metadata for a static file.
pub struct StaticFileMetadata {
    mime: Option<String>,
    size: u64,
    modified: std::time::SystemTime,
    etag: Option<String>,
}

impl StaticFileMetadata {
    /// Creates a new builder.
    ///
    /// # Returns
    ///
    /// * `StaticFileMetadataBuilder` - The new builder.
    pub fn builder() -> StaticFileMetadataBuilder {
        StaticFileMetadataBuilder {
            mime: None,
            size: 0,
            modified: std::time::SystemTime::now(),
            etag: None,
        }
    }

    /// Returns the mime type.
    ///
    /// # Returns
    ///
    /// * `Option<&String>` - The mime type.
    pub fn mime(&self) -> Option<&String> {
        self.mime.as_ref()
    }

    /// Returns the size.
    ///
    /// # Returns
    ///
    /// * `u64` - The size.
    pub fn size(&self) -> u64 {
        self.size
    }

    /// Returns the modified time.
    ///
    /// # Returns
    ///
    /// * `std::time::SystemTime` - The modified time.
    pub fn modified(&self) -> std::time::SystemTime {
        self.modified
    }

    /// Returns the etag.
    ///
    /// # Returns
    ///
    /// * `Option<&String>` - The etag.
    pub fn etag(&self) -> Option<&String> {
        self.etag.as_ref()
    }
}

#[derive(Clone)]
/// A static file that can be either in memory or on disk.
pub enum StaticFile {
    /// Data stored in memory.
    Data {
        /// The data.
        data: Vec<u8>,
        /// The metadata.
        metadata: StaticFileMetadata,
    },
    /// Data stored on disk.
    File {
        /// The path to the file.
        path: PathBuf,
        /// The metadata.
        metadata: StaticFileMetadata,
    },
}

impl StaticFile {
    /// Returns the metadata.
    ///
    /// # Returns
    ///
    /// * `&StaticFileMetadata` - The metadata.
    pub fn metadata(&self) -> &StaticFileMetadata {
        match self {
            StaticFile::Data { metadata, .. } => metadata,
            StaticFile::File { metadata, .. } => metadata,
        }
    }

    /// Returns the data if it is in memory.
    ///
    /// # Returns
    ///
    /// * `Option<&Vec<u8>>` - The data if it is in memory.
    pub fn data(&self) -> Option<&Vec<u8>> {
        match self {
            StaticFile::Data { data, .. } => Some(data),
            StaticFile::File { .. } => None,
        }
    }

    /// Returns the path if it is on disk.
    ///
    /// # Returns
    ///
    /// * `Option<&PathBuf>` - The path if it is on disk.
    pub fn path(&self) -> Option<&PathBuf> {
        match self {
            StaticFile::Data { .. } => None,
            StaticFile::File { path, .. } => Some(path),
        }
    }
}
