use crate::{
    SubProtocol, WebSocket,
    proto::websocket::{Message, WebSocketExt, WebSocketRead, WebSocketWrite},
};
use hyper::upgrade::Upgraded;
use hyper_util::rt::TokioIo;
use std::{
    io,
    pin::Pin,
    task::{Context, Poll},
};
use tokio::io::{AsyncRead, AsyncReadExt as _, AsyncWrite, AsyncWriteExt as _, ReadBuf};
use vetis::{VetisResult, errors::VetisError};
use ws_framer::{WsFrame, WsRxFramer, WsTxFramer};

#[derive(Clone, Copy)]
/// WebSocket sub protocol
pub struct WebSocketProto;

impl Default for WebSocketProto
where
    Self: SubProtocol,
{
    fn default() -> Self {
        WebSocketProto
    }
}

impl SubProtocol for WebSocketProto {
    fn version(&self) -> &str {
        "13"
    }
}

unsafe impl Send for WebSocket<TokioIo<Upgraded>, WebSocketProto> {}

impl From<TokioIo<Upgraded>> for WebSocket<TokioIo<Upgraded>, WebSocketProto> {
    fn from(value: TokioIo<Upgraded>) -> Self {
        Self { inner: value, proto: WebSocketProto::default().into() }
    }
}

impl WebSocketRead for WebSocket<TokioIo<Upgraded>, WebSocketProto> {
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

impl WebSocketWrite for WebSocket<TokioIo<Upgraded>, WebSocketProto> {
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

impl WebSocketExt for WebSocket<TokioIo<Upgraded>, WebSocketProto> {
    /// Sends a close frame to the WebSocket.
    ///
    /// # Arguments
    ///
    /// * `code` - The close code.
    /// * `reason` - The close reason.
    ///
    /// # Returns
    ///
    /// A Result indicating success or a DeboaExtrasError.
    async fn send_close(&mut self, code: u16, reason: &str) -> VetisResult<()> {
        self.write_message(Message::Close(code, reason.to_string()))
            .await
    }

    /// Sends a text frame to the WebSocket.
    ///
    /// # Arguments
    ///
    /// * `message` - The text message to send.
    ///
    /// # Returns
    ///
    /// A Result indicating success or a DeboaExtrasError.
    async fn send_text(&mut self, message: &str) -> VetisResult<()> {
        self.write_message(Message::Text(message.to_string()))
            .await
    }

    /// Sends a binary frame to the WebSocket.
    ///
    /// # Arguments
    ///
    /// * `message` - The binary message to send.
    ///
    /// # Returns
    ///
    /// A Result indicating success or a DeboaError.
    ///
    /// # Panics
    ///
    /// This function may panic if the WebSocket frame processing fails.
    ///
    async fn send_binary(&mut self, message: &[u8]) -> VetisResult<()> {
        self.write_message(Message::Binary(message.to_vec()))
            .await
    }

    /// Sends a ping frame to the WebSocket.
    ///
    /// # Arguments
    ///
    /// * `message` - The ping message to send.
    ///
    /// # Returns
    ///
    /// A Result indicating success or a DeboaError.
    ///
    /// # Panics
    ///
    /// This function may panic if the WebSocket frame processing fails.
    ///
    async fn send_ping(&mut self, message: &[u8]) -> VetisResult<()> {
        self.write_message(Message::Ping(message.to_vec()))
            .await
    }

    /// Sends a pong frame to the WebSocket.
    ///
    /// # Arguments
    ///
    /// * `message` - The pong message to send.
    ///
    /// # Returns
    ///
    /// # Panics
    ///
    /// This function may panic if the WebSocket frame processing fails.
    ///
    async fn send_pong(&mut self, message: &[u8]) -> VetisResult<()> {
        self.write_message(Message::Pong(message.to_vec()))
            .await
    }
}

impl AsyncRead for WebSocket<TokioIo<Upgraded>, WebSocketProto> {
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

impl AsyncWrite for WebSocket<TokioIo<Upgraded>, WebSocketProto> {
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
