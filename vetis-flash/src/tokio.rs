use crate::FlashPathConfig;
use hyper_body_utils::HttpBody;
use std::path::PathBuf;
use tokio::fs;
use upon::value;
use vetis::{
    Request, Response, VetisFutureResult,
    errors::{ContentError, HostError, VetisError},
    host::{HostContext, path::Path},
};

/// Static path
pub struct FlashPath {
    config: FlashPathConfig,
}

impl FlashPath {
    /// Create a new static path with provided configuration
    ///
    /// # Arguments
    ///
    /// * `config` - The configuration for the static path
    ///
    /// # Returns
    ///
    /// * `StaticPath` - The static path
    pub fn new(config: FlashPathConfig) -> FlashPath {
        FlashPath { config }
    }
}

impl Path for FlashPath {
    /// Returns the uri of flash path
    ///
    /// # Returns
    ///
    /// * `&str` - The uri of flash path
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
        _request: Request,
        host_context: HostContext,
    ) -> VetisFutureResult<'a, Response> {
        Box::pin(async move {
            if let Some(response) = self
                .config
                .response()
            {
                return Ok(Response::builder()
                    .status(http::StatusCode::OK)
                    .body(HttpBody::from_text(response)));
            } else if let Some(template) = self
                .config
                .template()
            {
                let template_path = std::path::Path::new(template);
                let full_template_path = if template_path.is_relative() {
                    if let Some(root_dir) = host_context.root_directory() {
                        PathBuf::from_iter([root_dir, template_path])
                    } else {
                        return Err(VetisError::Host(HostError::Content(ContentError::NotFound(
                            "Root directory undefined".to_string(),
                        ))));
                    }
                } else {
                    template_path.to_path_buf()
                };

                let template = fs::read_to_string(full_template_path)
                    .await
                    .map_err(|e| {
                        VetisError::Host(HostError::Content(ContentError::ServerError(
                            e.to_string(),
                        )))
                    })?;

                let mut engine = upon::Engine::new();
                engine
                    .add_template("main", template)
                    .map_err(|e| {
                        VetisError::Host(HostError::Content(ContentError::ServerError(
                            e.to_string(),
                        )))
                    })?;

                let output = engine
                    .template("main")
                    .render(value! {})
                    .to_string()
                    .map_err(|e| {
                        VetisError::Host(HostError::Content(ContentError::ServerError(
                            e.to_string(),
                        )))
                    })?;

                return Ok(Response::builder()
                    .status(http::StatusCode::OK)
                    .body(HttpBody::from_text(&output)));
            }

            Err(VetisError::Host(HostError::Content(ContentError::NotFound(
                "Invalid flash content, must be either response or template.".into(),
            ))))
        })
    }
}
