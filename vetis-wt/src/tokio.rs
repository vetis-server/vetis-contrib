use hyper::{
    Method, StatusCode,
    header::{self, CONNECTION, HeaderValue},
};
use hyper_util::rt::TokioIo;
use vetis::{
    Request, Response, Str, VetisFutureResult,
    errors::{HandlerError, HostError, VetisError},
    host::{HostContext, path::Path},
};

/// Builder for handler path
pub struct WebTransportPathBuilder {
    uri: Str,
}

impl WebTransportPathBuilder {
    /// Allow set handler uri path
    ///
    /// # Arguments
    ///
    /// * `uri` - The uri of the handler path
    ///
    /// # Returns
    ///
    /// * `Self` - The builder
    pub fn uri(mut self, uri: &str) -> Self {
        self.uri = Str::from(uri.to_string());
        self
    }

    /// Build the handler path
    ///
    /// # Returns
    ///
    /// * `Result<HandlerPath, VetisError>` - The handler path or error
    pub fn build(self) -> Result<WebTransportPath, VetisError> {
        if self.uri.is_empty() {
            return Err(VetisError::Host(HostError::Handler(HandlerError::Uri(
                "URI cannot be empty".to_string(),
            ))));
        }

        Ok(WebTransportPath { uri: self.uri.into() })
    }
}

/// WebTransport path
pub struct WebTransportPath {
    uri: Str,
}

impl WebTransportPath {
    /// Allow create a new handler path builder
    ///
    /// # Returns
    ///
    /// * `WebTransportPathBuilder` - The builder
    pub fn builder() -> WebTransportPathBuilder {
        WebTransportPathBuilder { uri: "/".into() }
    }
}

impl Path for WebTransportPath {
    fn uri(&self) -> &str {
        &self.uri
    }

    fn handle<'a>(
        &'a self,
        mut request: Request,
        host_context: HostContext,
    ) -> VetisFutureResult<'a, Response>
    where
        Self: Send,
    {
        let logger = host_context
            .logger()
            .clone();

        let headers = request.headers();
        let upgrade_header = headers.get(header::UPGRADE);
        if request.method() != Method::CONNECT {
            let response = Response::builder()
                .status(StatusCode::BAD_REQUEST)
                .empty();

            return Box::pin(async move { Ok(response) });
        }

        tokio::task::spawn(async move {});

        let future = async move {
            let response = Response::builder().empty();
            Ok(response)
        };

        Box::pin(future)
    }
}
