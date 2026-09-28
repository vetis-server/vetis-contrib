use crate::{
    RhaiPathConfig,
    request::RhaiRequest,
    response::{self, RhaiResponse},
};
use http::{HeaderMap, HeaderName, HeaderValue, StatusCode};
use log::info;
use rhai::{Engine, Scope};
use std::{num::TryFromIntError, path::PathBuf, str::FromStr, sync::Arc};
use vetis::{
    Request, Response, VetisFutureResult,
    errors::{ContentError, HostError, VetisError},
    host::{HostContext, path::Path},
};

/// Rhai path
pub struct RhaiPath {
    config: RhaiPathConfig,
}

impl RhaiPath {
    /// Create a new static path with provided configuration
    ///
    /// # Arguments
    ///
    /// * `config` - The configuration for the static path
    ///
    /// # Returns
    ///
    /// * `StaticPath` - The static path
    pub fn new(config: RhaiPathConfig) -> RhaiPath {
        RhaiPath { config }
    }
}

impl Path for RhaiPath {
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
    /// * `VetisFutureResult<'a, Response>` - The response to the request
    fn handle<'a>(
        &'a self,
        request: Request,
        host_context: HostContext,
    ) -> VetisFutureResult<'a, Response> {
        Box::pin(async move {
            let script_path = PathBuf::from(self.config.script());
            let script_path = if script_path.is_relative()
                && let Some(root_dir) = host_context.root_directory()
            {
                root_dir.join(&script_path)
            } else {
                script_path.to_path_buf()
            };

            // TODO: Move to a worker task or thread
            let mut engine = Engine::new();
            engine
                .register_type_with_name::<RhaiResponse>("RhaiResponse")
                .register_fn("new_response", RhaiResponse::new)
                .register_fn("add_header", response::add_header)
                .register_fn("write", response::write)
                .register_get("headers", RhaiResponse::get_headers)
                .register_get_set(
                    "status_code",
                    RhaiResponse::get_status_code,
                    RhaiResponse::set_status_code,
                );

            let ast = engine
                .compile_file(script_path)
                .map_err(|e| {
                    info!("Error: {}", e.to_string());
                    VetisError::Handler(format!("Couldt not compile script: {}", e.to_string()))
                })?;

            let mut rhai_request = RhaiRequest::new(
                &request
                    .uri()
                    .to_string(),
            );
            rhai_request.set_method(
                &request
                    .method()
                    .to_string(),
            );
            rhai_request.set_version(&format!("{:?}", request.version()));

            let engine = Arc::new(engine);
            let ast = Arc::new(ast);
            let request = Arc::new(rhai_request);

            let engine = engine.clone();
            let ast = ast.clone();
            let request = request.clone();

            let result = tokio::task::spawn_blocking(move || {
                let mut scope = Scope::new();
                scope.push("request", request);
                engine.eval_ast_with_scope::<RhaiResponse>(&mut scope, &ast)
            })
            .await
            .map_err(|e| {
                VetisError::Handler(format!("Could not execute script: {}", e.to_string()))
            })?;

            if let Ok(mut response) = result {
                let header_map = response
                    .get_headers()
                    .iter()
                    .try_fold(HeaderMap::default(), |mut acc, (name, value)| {
                        let header_name = HeaderName::from_str(name)
                            .map_err(|e| VetisError::Handler(e.to_string()))?;
                        let header_value = HeaderValue::from_str(&value.to_string())
                            .map_err(|e| VetisError::Handler(e.to_string()))?;
                        acc.insert(header_name, header_value);
                        Ok::<HeaderMap, VetisError>(acc)
                    })?;

                let status_code = response
                    .get_status_code()
                    .try_into()
                    .map_err(|e: TryFromIntError| VetisError::Handler(e.to_string()))
                    .and_then(|status_code| {
                        StatusCode::from_u16(status_code)
                            .map_err(|e| VetisError::Handler(e.to_string()))
                    })?;

                let response = Response::builder()
                    .status(status_code)
                    .headers(header_map)
                    .bytes(&response.get_body());
                Ok(response)
            } else {
                info!(
                    "Error: {}",
                    result
                        .err()
                        .unwrap()
                        .to_string()
                );
                Err(VetisError::Host(HostError::Content(ContentError::NotFound(
                    "Missing script".into(),
                ))))
            }
        })
    }
}
