use crate::{SubProtocol, WebSocket, WebSocketClient, WebSocketClientFactory};
use base64::{Engine, engine::general_purpose::STANDARD};
use hyper::{
    Method, StatusCode,
    header::{self, CONNECTION, HeaderValue, SEC_WEBSOCKET_ACCEPT, UPGRADE},
    upgrade::Upgraded,
};
use hyper_util::rt::TokioIo;
use std::sync::Arc;
use vetis::{
    Request, Response, VetisFutureResult, error,
    errors::{HandlerError, HostError, VetisError},
    host::{HostContext, path::Path},
};

/// IO Module
pub mod io;

const UPGRADE_WEBSOCKETS: &str = "websockets";
const CONNECTION_UPGRADE: &str = "upgrade";

/// Builder for handler path
pub struct WebSocketPathBuilder<P, F> {
    uri: Arc<str>,
    factory: F,
    sub_proto: P,
}

impl<P, F> WebSocketPathBuilder<P, F>
where
    P: SubProtocol,
    F: WebSocketClientFactory,
{
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
        self.uri = Arc::from(uri.to_string());
        self
    }

    /// Allow set factory function
    ///
    /// # Arguments
    ///
    /// * `factory` - The factory function
    ///
    /// # Returns
    ///
    /// * `Self` - The builder
    pub fn factory(mut self, factory: F) -> Self {
        self.factory = factory;
        self
    }

    /// Allow set sub proto
    ///
    /// # Arguments
    ///
    /// * `proto` - The subproto provider
    ///
    /// # Returns
    ///
    /// * `Self` - The builder
    pub fn sub_proto(mut self, proto: P) -> Self {
        self.sub_proto = proto.into();
        self
    }

    /// Build the handler path
    ///
    /// # Returns
    ///
    /// * `Result<HandlerPath, VetisError>` - The handler path or error
    pub fn build(self) -> Result<WebSocketPath<P, F>, VetisError> {
        if self.uri.is_empty() {
            return Err(VetisError::Host(HostError::Handler(HandlerError::Uri(
                "URI cannot be empty".to_string(),
            ))));
        }

        Ok(WebSocketPath { uri: self.uri.into(), factory: self.factory, sub_proto: self.sub_proto })
    }
}

/// WebSocket path
pub struct WebSocketPath<P, F> {
    uri: Arc<str>,
    factory: F,
    sub_proto: P,
}

impl<P, F> WebSocketPath<P, F>
where
    P: SubProtocol + Default,
    F: WebSocketClientFactory + Default,
{
    /// Allow create a new handler path builder
    ///
    /// # Returns
    ///
    /// * `WebSocketPathBuilder` - The builder
    pub fn builder() -> WebSocketPathBuilder<P, F> {
        WebSocketPathBuilder {
            uri: "/".into(),
            factory: F::default(),
            sub_proto: P::default().into(),
        }
    }
}

impl<P, F> Path for WebSocketPath<P, F>
where
    P: SubProtocol + Clone + Send + 'static,
    F: WebSocketClientFactory + Default,
    F::WebSocketClient:
        WebSocketClient<WebSocket = WebSocket<TokioIo<Upgraded>, P>> + Send + 'static,
{
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
        let proto = self
            .sub_proto
            .clone();

        let headers = request.headers();
        let upgrade_header = headers.get(header::UPGRADE);
        let version_header = headers.get(header::SEC_WEBSOCKET_VERSION);
        let key_header = headers.get(header::SEC_WEBSOCKET_KEY);
        if request.method() != Method::CONNECT
            && let Some(upgrade_header) = upgrade_header
            && let Ok(upgrade_type) = upgrade_header.to_str()
            && upgrade_type != UPGRADE_WEBSOCKETS
            && let Some(version_header) = version_header
            && let Ok(websocket_version) = version_header.to_str()
            && websocket_version != proto.version()
            && let Some(key_header) = key_header
            && let Ok(websocket_key) = key_header.to_str()
            && websocket_key.is_empty()
        {
            let response = Response::builder()
                .status(StatusCode::BAD_REQUEST)
                .empty();

            return Box::pin(async move { Ok(response) });
        }

        // Safe to do because of above
        let websocket_key = key_header
            .expect("Missing websocket key header")
            .to_str()
            .expect("Invalid websocket key")
            .to_string();

        let client = self
            .factory
            .create_client();
        tokio::task::spawn(async move {
            match hyper::upgrade::on(request.inner_mut()).await {
                Ok(upgraded) => {
                    client
                        .run(WebSocket::new(TokioIo::new(upgraded), proto))
                        .await
                }
                Err(e) => {
                    error!(logger, "WebSocket connection error: {}", e.to_string());
                    Err(VetisError::WebSocket(e.to_string()))
                }
            }
        });

        let future = async move {
            let accept_key =
                STANDARD.encode(format!("{}258EAFA5-E914-47DA-95CA-C5AB0DC85B11", websocket_key));

            let upgrade_websockets = HeaderValue::from_str(UPGRADE_WEBSOCKETS)
                .map_err(|e| VetisError::WebSocket(e.to_string()))?;
            let connection_upgrade = HeaderValue::from_str(CONNECTION_UPGRADE)
                .map_err(|e| VetisError::WebSocket(e.to_string()))?;
            let websocket_accept = HeaderValue::from_str(&accept_key)
                .map_err(|e| VetisError::WebSocket(e.to_string()))?;

            let response = Response::builder()
                .status(StatusCode::SWITCHING_PROTOCOLS)
                .header(UPGRADE, upgrade_websockets)
                .header(CONNECTION, connection_upgrade)
                .header(SEC_WEBSOCKET_ACCEPT, websocket_accept)
                .empty();
            Ok(response)
        };

        Box::pin(future)
    }
}
