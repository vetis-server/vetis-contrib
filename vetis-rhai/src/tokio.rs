use crate::{
    RhaiPathConfig,
    request::{self, ArchivedRequest},
    response,
};
use crossfire::{
    MRx,
    mpmc::{self, Array},
    oneshot,
};
use papaya::HashMap;
use rhai::{AST, Engine, Scope};
use rkyv::{rancor, util::AlignedVec};
use std::{path::PathBuf, sync::Arc};
use vetis::{
    Request, Response, VetisFutureResult, VetisResult,
    errors::VetisError,
    host::{HostContext, path::Path},
    worker::{Worker, WorkerLink},
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

            /*
            let bytes =
                rkyv::to_bytes::<rancor::Error>(&script_request).expect("Failed to serialize");
            */
            Ok(Response::builder().empty())
        })
    }
}

///RhaiScriptWorker
pub struct RhaiScriptWorker {
    id: usize,
    engine: Engine,
    cache: HashMap<String, Arc<AST>>,
    receiver: MRx<Array<(AlignedVec, oneshot::TxOneshot<AlignedVec>)>>,
    link: WorkerLink<AlignedVec>,
}

impl RhaiScriptWorker {
    /// Creates a new instance of worker
    pub fn new(id: usize) -> Self {
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

        let (sender, receiver) = mpmc::bounded_async_blocking(20000);

        Self {
            id,
            engine: engine.into(),
            cache: HashMap::new().into(),
            receiver: receiver.into(),
            link: WorkerLink::new("rhai", sender),
        }
    }
}

impl Worker for RhaiScriptWorker {
    type MessageType = AlignedVec;

    fn id(&self) -> usize {
        self.id
    }

    fn link(&self) -> &WorkerLink<AlignedVec> {
        &self.link
    }

    fn run(&self) -> VetisResult<()> {
        while let Ok(raw_request) = self.receiver.recv() {
            let request = rkyv::access::<ArchivedRequest, rkyv::rancor::Error>(&raw_request.0)
                .expect("Could not receive request!");

            let script_path = request
                .script_path
                .to_string();
            let ast = if !self
                .cache
                .pin()
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
                self.cache
                    .pin()
                    .insert(script_path.clone(), new_ast);
                ret_ast
            } else {
                self.cache
                    .pin()
                    .get(&script_path.clone())
                    .unwrap()
                    .clone()
            };

            let request = request::Request::new(
                &request
                    .uri
                    .to_string(),
            );

            let mut scope = Scope::new();
            scope.push("request", request);
            let result = self
                .engine
                .eval_ast_with_scope::<response::Response>(&mut scope, &ast);

            let response = result.unwrap_or_else(|_f| {
                let mut response = response::Response::new();
                response.set_status_code(500);
                response
            });

            let bytes = rkyv::to_bytes::<rancor::Error>(&response).expect("Failed to serialize");

            raw_request
                .1
                .send(bytes);
        }

        Ok(())
    }
}
