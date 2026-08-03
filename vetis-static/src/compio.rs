use crate::{format_date, StaticFile, StaticFileMetadata, StaticPathConfig};
use compio::fs::File;
use http::{HeaderMap, HeaderValue};
use hyper_body_utils::HttpBody;
use log::error;
use send_wrapper::SendWrapper;
#[cfg(unix)]
use std::os::unix::fs::MetadataExt;
#[cfg(windows)]
use std::os::windows::fs::MetadataExt;
use std::{future::Future, path::PathBuf, pin::Pin, sync::Arc};
use vetis::{
    errors::{FileError, VetisError, VirtualHostError},
    virtual_host::path::Path,
    Request, Response, VetisResult,
};

/// Static path
pub struct StaticPath {
    config: StaticPathConfig,
    index_file: Option<String>,
    //file_cache: VetisFileCache,
}

impl StaticPath {
    /// Create a new static path with provided configuration
    ///
    /// # Arguments
    ///
    /// * `config` - The configuration for the static path
    ///
    /// # Returns
    ///
    /// * `StaticPath` - The static path
    pub fn new(config: StaticPathConfig) -> StaticPath {
        /*
        let file_cache = if let Some(cache) = config.cache() {
            CacheBuilder::new(cache.capacity())
                .time_to_idle(cache.tti())
                .time_to_live(cache.ttl())
        } else {
            CacheBuilder::new(1000)
                .time_to_idle(Duration::from_secs(60))
                .time_to_live(Duration::from_secs(60))
        }
        .build();
        */

        if let Some(index_files) = config.index_files() {
            let directory = PathBuf::from(config.directory());
            if let Some(index_file) = index_files
                .iter()
                .find(|index_file| {
                    directory
                        .join(index_file)
                        .exists()
                })
            {
                return StaticPath {
                    config: config.clone(),
                    index_file: Some(index_file.to_string()),
                    //file_cache,
                };
            }
        }
        StaticPath { config, index_file: None /*file_cache*/ }
    }

    async fn cache_file(&self, file_path: &std::path::Path) -> VetisResult<StaticFile> {
        let path = file_path
            .display()
            .to_string();

        let file = File::open(path.clone()).await;
        let file = match file {
            Ok(file) => {
                let metadata = match file
                    .metadata()
                    .await
                {
                    Ok(metadata) => metadata,
                    Err(e) => {
                        error!("Error getting metadata for file {:?}: {}", file_path, e);
                        return Err(VetisError::VirtualHost(VirtualHostError::File(
                            FileError::NotFound,
                        )));
                    }
                };

                let modified = metadata
                    .modified()
                    .unwrap_or(std::time::SystemTime::now());

                let file_name = file_path.file_name();

                let mime_type = match file_name {
                    Some(file_name) => match file_name.to_str() {
                        Some(file_name) => match minimime::lookup_by_filename(file_name) {
                            Some(mime) => Some(mime.content_type),
                            None => None,
                        },
                        None => None,
                    },
                    None => None,
                };

                let metadata = StaticFileMetadata {
                    mime: mime_type,
                    #[cfg(unix)]
                    size: metadata.size(),
                    #[cfg(windows)]
                    size: metadata.file_size(),
                    modified,
                    etag: None,
                };

                let max_file_size = if let Some(cache) = self.config.cache() {
                    cache.max_file_size() as u64
                } else {
                    1024 * 1024 * 10 // 10MB default
                };

                let static_file = if metadata.size() < max_file_size {
                    let data = compio::fs::read(file_path).await;
                    if let Ok(data) = data {
                        StaticFile::Data { data, metadata }
                    } else {
                        return Err(VetisError::VirtualHost(VirtualHostError::File(
                            FileError::NotFound,
                        )));
                    }
                } else {
                    StaticFile::File { path: file_path.to_path_buf(), metadata }
                };

                Ok(static_file)
            }
            Err(e) => {
                error!("Error opening file {}: {}", path, e);
                Err(VetisError::VirtualHost(VirtualHostError::File(FileError::NotFound)))
            }
        };

        file
    }

    async fn serve_file(
        &self,
        file_path: &std::path::Path,
        range: Option<&str>,
    ) -> VetisResult<Response> {
        let file = self
            .cache_file(file_path)
            .await?;

        let filesize = file
            .metadata()
            .size();

        if let Some(range) = range {
            let range_info = match range
                .split_once("=")
                .ok_or(VetisError::VirtualHost(VirtualHostError::File(FileError::InvalidRange)))
            {
                Ok(info) => info,
                Err(e) => return Err(e),
            };

            let (unit, range) = range_info;
            if unit != "bytes" {
                return Err(VetisError::VirtualHost(VirtualHostError::File(
                    FileError::InvalidRange,
                )));
            }

            let (start, end) = range
                .split_once("-")
                .ok_or(VetisError::VirtualHost(VirtualHostError::File(FileError::InvalidRange)))?;
            let start = start
                .parse::<u64>()
                .map_err(|_| {
                    VetisError::VirtualHost(VirtualHostError::File(FileError::InvalidRange))
                })?;
            let end = end
                .parse::<u64>()
                .map_err(|_| {
                    VetisError::VirtualHost(VirtualHostError::File(FileError::InvalidRange))
                })?;
            if start > end || start >= filesize {
                return Ok(Response::builder()
                    .status(http::StatusCode::RANGE_NOT_SATISFIABLE)
                    .body(HttpBody::from_text("")));
            } else if start < end && end < filesize {
                return Ok(Response::builder()
                    .status(http::StatusCode::PARTIAL_CONTENT)
                    .body(HttpBody::from_bytes(file.data().unwrap())));
            }
        }

        Ok(Response::builder()
            .status(http::StatusCode::OK)
            .header(
                http::header::ACCEPT_RANGES,
                "bytes"
                    .parse()
                    .unwrap(),
            )
            .header(http::header::CONTENT_LENGTH, HeaderValue::from(filesize))
            .body(HttpBody::from_bytes(file.data().unwrap())))
    }

    async fn serve_metadata(&self, file_path: PathBuf) -> VetisResult<Response> {
        let file = self
            .cache_file(&file_path)
            .await?;

        let len = file
            .metadata()
            .size();
        let mut headers = HeaderMap::new();
        match len
            .to_string()
            .parse()
        {
            Ok(len) => {
                headers.insert(http::header::CONTENT_LENGTH, len);
            }
            Err(_) => todo!(),
        }
        let last_modified = file
            .metadata()
            .modified();
        let date = format_date(last_modified);
        headers.insert(
            http::header::LAST_MODIFIED,
            date.parse()
                .map_err(|_| {
                    VetisError::VirtualHost(VirtualHostError::File(FileError::InvalidMetadata))
                })?,
        );

        let mime_type = file
            .metadata()
            .mime();
        if let Some(mime_type) = mime_type {
            headers.insert(
                http::header::CONTENT_TYPE,
                HeaderValue::from_str(mime_type).map_err(|_| {
                    VetisError::VirtualHost(VirtualHostError::File(FileError::InvalidMetadata))
                })?,
            );
        }

        let response = Response::builder()
            .status(http::StatusCode::OK)
            .headers(headers)
            .text("");

        Ok(response)
    }

    async fn serve_index_file(&self, directory: &std::path::Path) -> VetisResult<Response> {
        match &self.index_file {
            Some(index_file) => {
                let full_path = directory.join(index_file);
                self.serve_file(&full_path, None)
                    .await
            }
            None => {
                println!("No index file configured");
                Err(VetisError::VirtualHost(VirtualHostError::File(FileError::NotFound)))
            }
        }
    }
}

impl Path for StaticPath {
    /// Returns the uri of the static path
    ///
    /// # Returns
    ///
    /// * `&str` - The uri of the static path
    fn uri(&self) -> &str {
        self.config.uri()
    }

    /// Handles the request for the static path
    ///
    /// # Returns
    ///
    /// * `Pin<Box<dyn Future<Output = Result<Response, VetisError>> + Send + '_>>` - The response to the request
    fn handle<'a>(
        &'a self,
        request: Request,
        uri: Arc<String>,
    ) -> Pin<Box<dyn Future<Output = Result<Response, VetisError>> + Send + 'a>> {
        let future = async move {
            let ext_regex = regex::Regex::new(
                self.config
                    .extensions(),
            );

            let directory = PathBuf::from(
                self.config
                    .directory(),
            );

            let uri = uri
                .strip_prefix("/")
                .unwrap_or(&uri);
            let file = directory.join(uri);

            if self
                .config
                .index_files()
                .is_some()
            {
                if !file.exists() {
                    if let Ok(ext_regex) = ext_regex {
                        if !ext_regex.is_match(uri.as_ref()) {
                            return Err(VetisError::VirtualHost(VirtualHostError::File(
                                FileError::NotFound,
                            )));
                        }
                    }
                } else if file.is_dir() {
                    return self
                        .serve_index_file(&file)
                        .await;
                }
            } else if !file.exists() {
                return Err(VetisError::VirtualHost(VirtualHostError::File(FileError::NotFound)));
            }

            if request.method() == http::Method::HEAD {
                return self
                    .serve_metadata(file)
                    .await;
            }

            let range = if request
                .headers()
                .contains_key(http::header::RANGE)
            {
                let value = request
                    .headers()
                    .get(http::header::RANGE);
                Some(
                    value
                        .unwrap()
                        .to_str()
                        .unwrap(),
                )
            } else {
                None
            };

            self.serve_file(&file, range)
                .await
        };

        Box::pin(SendWrapper::new(future))
    }
}
