use crate::{StaticFile, StaticFileMetadata, StaticPathConfig, format_date};
use http::{HeaderMap, HeaderValue};
use hyper_body_utils::HttpBody;
#[cfg(unix)]
use std::os::unix::fs::MetadataExt;
#[cfg(windows)]
use std::os::windows::fs::MetadataExt;
use std::path::PathBuf;
use tokio::fs::File;
use vetis::{
    Request, Response, VetisFutureResult, VetisResult,
    errors::{ContentError, HostError, VetisError},
    host::{HostContext, path::Path},
};

/// Static path
pub struct StaticPath {
    config: StaticPathConfig,
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
        StaticPath { config }
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
                        let message =
                            format!("Error getting metadata for file {:?}: {}", file_path, e);
                        return Err(VetisError::Host(HostError::Content(ContentError::NotFound(
                            message,
                        ))));
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
                    let file_data = tokio::fs::read(file_path).await;
                    match file_data {
                        Ok(data) => StaticFile::Data { data, metadata },
                        Err(e) => {
                            return Err(VetisError::Host(HostError::Content(
                                ContentError::NotFound(e.to_string()),
                            )));
                        }
                    }
                } else {
                    StaticFile::File { path: file_path.to_path_buf(), metadata }
                };

                Ok(static_file)
            }
            Err(e) => {
                let message = format!("Error opening file {}: {}", path, e);
                Err(VetisError::Host(HostError::Content(ContentError::NotFound(message))))
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

        let mimetype = if let Some(mime) = file
            .metadata()
            .mime()
        {
            HeaderValue::from_bytes(mime.as_bytes())
        } else {
            HeaderValue::from_bytes(b"text/plain")
        }
        .map_err(|e| {
            VetisError::Host(HostError::Content(ContentError::InvalidMetadata(e.to_string())))
        })?;

        if let Some(range) = range {
            let range_info = match range
                .split_once("=")
                .ok_or(VetisError::Host(HostError::Content(ContentError::InvalidRange(
                    "Missing value".to_string(),
                )))) {
                Ok(info) => info,
                Err(e) => return Err(e),
            };

            let (unit, range) = range_info;
            if unit != "bytes" {
                return Err(VetisError::Host(HostError::Content(ContentError::InvalidRange(
                    "Only bytes ranges are allowed!".to_string(),
                ))));
            }

            let (start, end) = range
                .split_once("-")
                .ok_or(VetisError::Host(HostError::Content(ContentError::InvalidRange(
                    "Invalid format!".to_string(),
                ))))?;
            let start = start
                .parse::<u64>()
                .map_err(|e| {
                    VetisError::Host(HostError::Content(ContentError::InvalidRange(e.to_string())))
                })?;
            let end = end
                .parse::<u64>()
                .map_err(|e| {
                    VetisError::Host(HostError::Content(ContentError::InvalidRange(e.to_string())))
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
            .header(http::header::CONTENT_TYPE, mimetype)
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
            Err(e) => {
                return Err(VetisError::Host(HostError::Content(ContentError::InvalidMetadata(
                    e.to_string(),
                ))));
            }
        }
        let last_modified = file
            .metadata()
            .modified();
        let date = format_date(last_modified);
        headers.insert(
            http::header::LAST_MODIFIED,
            date.parse()
                .map_err(|e: http::header::InvalidHeaderValue| {
                    VetisError::Host(HostError::Content(ContentError::InvalidMetadata(
                        e.to_string(),
                    )))
                })?,
        );

        let mime_type = file
            .metadata()
            .mime();
        if let Some(mime_type) = mime_type {
            headers.insert(
                http::header::CONTENT_TYPE,
                HeaderValue::from_str(mime_type).map_err(|e| {
                    VetisError::Host(HostError::Content(ContentError::InvalidMetadata(
                        e.to_string(),
                    )))
                })?,
            );
        }

        let response = Response::builder()
            .status(http::StatusCode::OK)
            .headers(headers)
            .empty();

        Ok(response)
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
        host_context: HostContext,
    ) -> VetisFutureResult<'a, Response> {
        Box::pin(async move {
            let ext_regex = regex::Regex::new(
                self.config
                    .extensions(),
            );

            let directory = PathBuf::from(
                self.config
                    .directory(),
            );

            let uri = host_context
                .path_uri()
                .strip_prefix("/")
                .unwrap_or(&host_context.path_uri());

            let static_dir = if directory.is_relative()
                && let Some(root_dir) = host_context.root_directory()
            {
                root_dir.join(&directory)
            } else {
                directory.to_path_buf()
            };

            let file = static_dir.join(uri);
            if let Some(index_files) = self
                .config
                .index_files()
                && file.is_dir()
            {
                for index in index_files {
                    let path_to_index = static_dir.join(index);
                    if path_to_index.exists() {
                        return self
                            .serve_file(&path_to_index, None)
                            .await;
                    }
                }
                return Err(VetisError::Host(HostError::Content(ContentError::Forbidden)));
            }

            if !file.exists() {
                return Err(VetisError::Host(HostError::Content(ContentError::NotFound(
                    "File does not exist".to_string(),
                ))));
            }

            if let Ok(ext_regex) = ext_regex
                && !ext_regex.is_match(uri.as_ref())
            {
                return Err(VetisError::Host(HostError::Content(ContentError::Forbidden)));
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
                if let Some(header) = request
                    .headers()
                    .get(http::header::RANGE)
                {
                    match header.to_str() {
                        Ok(v) => Some(v),
                        Err(_) => None,
                    }
                } else {
                    None
                }
            } else {
                None
            };

            self.serve_file(&file, range)
                .await
        })
    }
}
