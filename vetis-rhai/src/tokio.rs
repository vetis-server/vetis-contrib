use crate::{
    RhaiPathConfig,
    request::{self},
    response,
};
use http::{HeaderMap, HeaderName, HeaderValue, StatusCode};
use rhai::{AST, Engine, Scope};
use std::{
    collections::HashMap,
    num::TryFromIntError,
    path::PathBuf,
    str::FromStr as _,
    sync::{Arc, LazyLock},
};
use tokio::sync::RwLock;
use vetis::{
    Request, Response, VetisFutureResult,
    errors::VetisError,
    host::{HostContext, path::Path},
};

static SCRIPT_CACHE: LazyLock<RwLock<HashMap<PathBuf, Arc<AST>>>> =
    LazyLock::new(|| RwLock::new(HashMap::new()));

/// Rhai path
pub struct RhaiPath {
    config: RhaiPathConfig,
    engine: Arc<Engine>,
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
        let mut engine = Engine::new();
        engine
            .register_type_with_name::<response::Response>("RhaiResponse")
            .register_fn("new_response", response::Response::new)
            .register_fn("add_header", response::add_header)
            .register_fn("write", response::write)
            .register_get("headers", response::Response::get_headers)
            .register_get_set(
                "status_code",
                response::Response::get_status_code,
                response::Response::set_status_code,
            );

        RhaiPath { config, engine: engine.into() }
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
                if root_dir.is_relative()
                    && let Ok(current_dir) = std::env::current_dir()
                {
                    current_dir
                        .join(root_dir)
                        .join(&script_path)
                } else {
                    root_dir.join(&script_path)
                }
            } else {
                script_path.to_path_buf()
            };

            let mut script_request = request::Request::new(
                &request
                    .uri()
                    .to_string(),
            );
            script_request.set_script_path(
                script_path
                    .to_str()
                    .unwrap(),
            );
            script_request.set_method(
                &request
                    .method()
                    .to_string(),
            );

            let request = Arc::new(request);

            let ast = if !SCRIPT_CACHE
                .read()
                .await
                .contains_key(&script_path)
            {
                let ast = self
                    .engine
                    .compile_file(
                        script_path
                            .clone()
                            .into(),
                    )
                    .map_err(|e| {
                        //info!("Error: {}", e.to_string());
                        VetisError::Worker(format!("Couldt not compile script: {}", e.to_string()))
                    })?;

                let new_ast = Arc::new(ast);
                let ret_ast = new_ast.clone();

                SCRIPT_CACHE
                    .write()
                    .await
                    .insert(script_path.clone(), new_ast);

                ret_ast
            } else {
                SCRIPT_CACHE
                    .read()
                    .await
                    .get(&script_path)
                    .unwrap()
                    .clone()
            };

            let engine = self.engine.clone();
            let ast = ast.clone();

            let result = tokio::spawn(async move {
                let mut scope = Scope::new();
                scope.push("request", request.clone());
                engine.eval_ast_with_scope::<response::Response>(&mut scope, &ast)
            })
            .await
            .map_err(|e| {
                VetisError::Handler(format!("Could not execute script: {}", e.to_string()))
            })?;

            let mut response = result.unwrap_or_else(|_f| {
                let mut response = response::Response::new();
                response.set_status_code(500);
                response
            });

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
        })
    }
}
