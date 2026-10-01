#![doc = include_str!("../README.md")]
#![deny(missing_docs)]
use pin_project_lite::pin_project;
use vetis::VetisResult;

/// Proto module
pub mod proto;
#[cfg(feature = "runtime-tokio")]
/// Tokio runtime support module
pub mod tokio;

/// WebSocket client trait
pub trait WebSocketClient {
    /// WebSocket type
    type WebSocket;
    /// Run method
    fn run(&self, socket: Self::WebSocket) -> impl Future<Output = VetisResult<()>> + Send;
}

/// WebSocket client factory trait
pub trait WebSocketClientFactory {
    /// WebSocket client type
    type WebSocketClient: WebSocketClient + Default;

    /// Create client method
    fn create_client(&self) -> Self::WebSocketClient {
        Self::WebSocketClient::default()
    }
}

pin_project! {
    /// WebSocket struct
    pub struct WebSocket<T, P>
    {
        #[pin]
        pub(crate) inner: T,
        proto: P,
    }
}

impl<T, P> WebSocket<T, P>
where
    P: SubProtocol,
{
    /// new method
    ///
    /// # Arguments
    ///
    /// * `inner` - A inner stream.
    ///
    /// # Returns
    ///
    /// A WebSocket struct.
    ///
    pub fn new(inner: T, proto: P) -> Self {
        Self { inner, proto }
    }
}

/// SubProtocol trait
pub trait SubProtocol {
    /// Returns the sub protocol version
    fn version(&self) -> &str;
}
