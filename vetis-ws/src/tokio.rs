use crate::{Message, WebSocket, WebSocketRead, WebSocketWrite};
use hyper::{
    Method, StatusCode,
    header::{self, CONNECTION, HeaderValue, SEC_WEBSOCKET_ACCEPT, UPGRADE},
    upgrade::Upgraded,
};
use hyper_util::rt::TokioIo;
use std::{
    io,
    pin::Pin,
    sync::Arc,
    task::{Context, Poll},
};
use tokio::io::{AsyncRead, AsyncReadExt as _, AsyncWrite, AsyncWriteExt as _, ReadBuf};
use vetis::{
    Request, Response, VetisFutureResult, VetisResult, error,
    errors::{HandlerError, HostError, VetisError},
    host::{HostContext, path::Path},
};
use ws_framer::{WsFrame, WsRxFramer, WsTxFramer};

const UPGRADE_WEBSOCKETS: &str = "websockets";
const CONNECTION_UPGRADE: &str = "upgrade";

/// Type alias for boxed handler closures.
///
/// This represents an async function that takes a `Request` and returns
/// a `Response` or an error. Handlers are the core of request processing
/// in VeTiS hosts.
///
/// # Examples
///
/// ```rust,no_run
/// use vetis::HandlerFn;
/// use vetis::{Request, Response, errors::VetisError};
///
/// let handler: HandlerFn = Box::new(|request: Request| {
///     Box::pin(async move {
///         // Process request...
///         Ok(Response::builder()
///             .status(http::StatusCode::OK)
///             .text("OK"))
///     })
/// });
/// ```
pub type HandlerFn =
    Box<dyn Fn(WebSocket<TokioIo<Upgraded>>) -> VetisFutureResult<'static, Response> + Send + Sync>;

/// Creates a handler function from a function.
///
/// This utility function converts any compatible async function into a
/// `HandlerFn` that can be used with hosts.
///
/// # Arguments
///
/// * `f` - An async function that takes a `Request` and returns a `VetisResult<Response>`
///
/// # Examples
///
/// ```rust,no_run
/// use vetis::{
///     host::{handler_fn, HostConfig},
/// };
///
/// let config = HostConfig::builder()
///     .hostname("example.com")
///     .build()
///     .unwrap();
///
/// assert_eq!("example.com", config.hostname());
/// ```
pub fn handler_fn<F, Fut>(f: F) -> HandlerFn
where
    F: Fn(WebSocket<TokioIo<Upgraded>>) -> Fut + Send + Sync + 'static,
    Fut: Future<Output = VetisResult<Response>> + Send + Sync + 'static,
{
    Box::new(move |stream| Box::pin(f(stream)))
}

/// Builder for handler path
pub struct WebSocketPathBuilder {
    uri: Arc<str>,
    handler: Option<HandlerFn>,
}

impl WebSocketPathBuilder {
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

    /// Allow set handler function
    ///
    /// # Arguments
    ///
    /// * `handler` - The handler function
    ///
    /// # Returns
    ///
    /// * `Self` - The builder
    pub fn handler(mut self, handler: HandlerFn) -> Self {
        self.handler = Some(handler);
        self
    }

    /// Build the handler path
    ///
    /// # Returns
    ///
    /// * `Result<HandlerPath, VetisError>` - The handler path or error
    pub fn build(self) -> Result<WebSocketPath, VetisError> {
        if self.uri.is_empty() {
            return Err(VetisError::Host(HostError::Handler(HandlerError::Uri(
                "URI cannot be empty".to_string(),
            ))));
        }

        let handler = match self.handler {
            Some(handler) => handler,
            None => {
                return Err(VetisError::Host(HostError::Handler(HandlerError::Handler(
                    "Handler must be set".to_string(),
                ))));
            }
        };

        Ok(WebSocketPath { uri: self.uri.into(), handler: handler.into() })
    }
}

/// WebSocket path
pub struct WebSocketPath {
    uri: Arc<str>,
    handler: Arc<HandlerFn>,
}

unsafe impl Send for WebSocketPath {}
unsafe impl Sync for WebSocketPath {}

impl WebSocketPath {
    /// Allow create a new handler path builder
    ///
    /// # Returns
    ///
    /// * `WebSocketPathBuilder` - The builder
    pub fn builder() -> WebSocketPathBuilder {
        WebSocketPathBuilder { uri: "/".into(), handler: None }
    }
}

impl Path for WebSocketPath {
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
        let handler = self.handler.clone();
        let future = async move {
            if request.method() != Method::CONNECT
                && let Some(upgrade) = request
                    .headers()
                    .get(header::UPGRADE)
                && let Ok(upgrade_type) = upgrade.to_str()
                && upgrade_type != UPGRADE_WEBSOCKETS
            {
                let response = Response::builder()
                    .status(StatusCode::BAD_REQUEST)
                    .empty();

                return Ok(response);
            }

            // Handle upgrades
            let logger = logger.clone();
            let handler = self.handler.clone();
            tokio::task::spawn(async move {
                match hyper::upgrade::on(request.inner_mut()).await {
                    Ok(upgraded) => (self.handler)(WebSocket::new(TokioIo::new(upgraded))),
                    Err(e) => {
                        error!(logger, "WebSocket connection error: {}", e.to_string())
                    }
                }
            });

            let upgrade_websockets = HeaderValue::from_str(UPGRADE_WEBSOCKETS)
                .map_err(|e| VetisError::WebSocket(e.to_string()))?;
            let connection_upgrade = HeaderValue::from_str(CONNECTION_UPGRADE)
                .map_err(|e| VetisError::WebSocket(e.to_string()))?;
            let websocket_accept = HeaderValue::from_str("your-key")
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

impl WebSocketRead for WebSocket<TokioIo<Upgraded>> {
    async fn read_message(&mut self) -> VetisResult<Option<Message>> {
        let mut rx_buf = vec![0; 10240];
        let mut rx_framer = WsRxFramer::new(&mut rx_buf);

        let bytes_read = self
            .inner
            .read(rx_framer.mut_buf())
            .await;
        if bytes_read.is_err() {
            return Err(VetisError::WebSocket("Failed to read message".to_string()));
        }

        let bytes_read = bytes_read.unwrap();
        rx_framer.revolve_write_offset(bytes_read);
        let res = rx_framer.process_data();
        let message = if let Some(frame) = res {
            #[allow(clippy::collapsible_match)]
            match frame {
                WsFrame::Text(data) => Some(Message::Text(data.to_string())),
                WsFrame::Binary(data) => Some(Message::Binary(data.to_vec())),
                WsFrame::Close(code, reason) => Some(Message::Close(code, reason.to_string())),
                WsFrame::Ping(data) => Some(Message::Ping(data.to_vec())),
                _ => None,
            }
        } else {
            None
        };

        Ok(message)
    }
}

impl WebSocketWrite for &mut WebSocket<TokioIo<Upgraded>> {
    async fn write_message(&mut self, message: Message) -> VetisResult<()> {
        let mut tx_buf = vec![0; 10240];
        let mut tx_framer = WsTxFramer::new(true, &mut tx_buf);

        let result = match message {
            Message::Text(data) => {
                self.write_all(tx_framer.frame(WsFrame::Text(&data)))
                    .await
            }
            Message::Binary(data) => {
                self.write_all(tx_framer.frame(WsFrame::Binary(&data)))
                    .await
            }
            Message::Close(code, reason) => {
                self.write_all(tx_framer.frame(WsFrame::Close(code, &reason)))
                    .await
            }
            Message::Ping(data) => {
                self.write_all(tx_framer.frame(WsFrame::Ping(&data)))
                    .await
            }
            _ => Ok(()),
        };

        if result.is_err() {
            return Err(VetisError::WebSocket("Failed to send frame".into()));
        }

        Ok(())
    }
}

impl AsyncRead for WebSocket<TokioIo<Upgraded>> {
    fn poll_read(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut ReadBuf<'_>,
    ) -> Poll<io::Result<()>> {
        self.project()
            .inner
            .poll_read(cx, buf)
    }
}

impl AsyncWrite for WebSocket<TokioIo<Upgraded>> {
    fn poll_write(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<io::Result<usize>> {
        self.project()
            .inner
            .poll_write(cx, buf)
    }

    fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        self.project()
            .inner
            .poll_flush(cx)
    }

    fn poll_shutdown(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        self.project()
            .inner
            .poll_shutdown(cx)
    }

    fn poll_write_vectored(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        bufs: &[std::io::IoSlice<'_>],
    ) -> Poll<io::Result<usize>> {
        let buf = bufs
            .iter()
            .find(|b| !b.is_empty())
            .map_or(&[][..], |b| &**b);
        self.project()
            .inner
            .poll_write(cx, buf)
    }

    fn is_write_vectored(&self) -> bool {
        self.inner
            .is_write_vectored()
    }
}
