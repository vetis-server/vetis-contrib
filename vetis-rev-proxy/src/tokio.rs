use crate::ReverseProxyPathConfig;
use deboa::{HttpClient as _, request::DeboaRequest};
use deboa_tokio::{Client, client::http::conn::pool::HttpConnectionPool};
use once_cell::sync::Lazy;
use vetis::{
    Request, Response, VetisFutureResult,
    errors::{HostError, VetisError},
    host::{HostContext, path::Path},
};

static CLIENT: Lazy<Client> = Lazy::new(|| {
    Client::builder()
        .connection_pool(HttpConnectionPool::default())
        .prior_knowledge(true)
        .build()
});

/// Proxy path
pub struct ReveseProxyPath {
    config: ReverseProxyPathConfig,
}

impl ReveseProxyPath {
    /// Create a new proxy path with provided configuration
    ///
    /// # Arguments
    ///
    /// * `config` - The proxy path configuration
    ///
    /// # Returns
    ///
    /// * `ProxyPath` - The proxy path
    pub fn new(config: ReverseProxyPathConfig) -> ReveseProxyPath {
        ReveseProxyPath { config }
    }
}

impl Path for ReveseProxyPath {
    /// Get the URI of the proxy path
    ///
    /// # Returns
    ///
    /// * `&str` - The URI of the proxy path
    fn uri(&self) -> &str {
        self.config.uri()
    }

    /// Handle proxy request
    ///
    /// # Arguments
    ///
    /// * `request` - The request to handle
    /// * `uri` - The URI of the request
    ///
    /// # Returns
    ///
    /// * `Pin<Box<dyn Future<Output = Result<Response, VetisError>> + Send + Sync + '_>>` - The future that will resolve to the response
    fn handle<'a>(
        &'a self,
        request: Request,
        host_context: HostContext,
    ) -> VetisFutureResult<'a, Response> {
        let (request_parts, request_body) = request.into_parts();

        let target = self.config.target();

        Box::pin(async move {
            let target_url = format!("{}{}", target, host_context.path_uri());
            let deboa_request = match DeboaRequest::at(target_url, request_parts.method) {
                Ok(request) => request,
                Err(e) => return Err(VetisError::Host(HostError::Proxy(e.to_string()))),
            };

            let deboa_request = match deboa_request
                .version(request_parts.version)
                .headers(request_parts.headers)
                .body(request_body)
                .build()
            {
                Ok(request) => request,
                Err(e) => return Err(VetisError::Host(HostError::Proxy(e.to_string()))),
            };

            // TODO: Check errors and handle them properly by returning a proper response 500, 503 or 504
            let response = CLIENT
                .execute(deboa_request)
                .await;

            let response = match response {
                Ok(response) => response,
                Err(e) => return Err(VetisError::Host(HostError::Proxy(e.to_string()))),
            };

            let (response_parts, response_body) = response.into_parts();

            let vetis_response = Response::builder()
                .status(response_parts.status)
                .headers(response_parts.headers)
                .body(response_body);

            Ok::<Response, VetisError>(vetis_response)
        })
    }
}
