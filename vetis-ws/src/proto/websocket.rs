use vetis::VetisResult;

/// Message enum
///
/// # Variants
///
/// * `Text(String)` - A text message.
/// * `Binary(Vec<u8>)` - A binary message.
/// * `Close(u16, String)` - A close message.
/// * `Ping(Vec<u8>)` - A ping message.
/// * `Pong(Vec<u8>)` - A pong message.
#[derive(Clone)]
pub enum Message {
    /// A text message
    Text(String),
    /// BBinary message
    Binary(Vec<u8>),
    /// Close message
    Close(u16, String),
    /// Ping message
    Ping(Vec<u8>),
    /// Pong reply message
    Pong(Vec<u8>),
}

/// Trait to allow read websocket messages
pub trait WebSocketRead {
    /// Reads a message from the WebSocket.
    ///
    /// # Returns
    ///
    /// A Result containing an Option<Message> or a DeboaExtrasError.
    ///
    /// # Examples
    ///
    /// ```rust, compile_fail
    /// while let Some(message) = websocket.read_message().await {
    ///     println!("message: {}", message);
    /// }
    /// ```
    ///
    /// # Panics
    ///
    /// This function may panic if the WebSocket frame processing fails.
    ///
    fn read_message(&mut self) -> impl Future<Output = VetisResult<Option<Message>>>;
}

/// Trait to allow write websocket messages
pub trait WebSocketWrite {
    /// Writes a message to the WebSocket.
    ///
    /// # Arguments
    ///
    /// * `message` - The message to write.
    ///
    /// # Returns
    ///
    /// A Result indicating success or a DeboaExtrasError.
    ///
    /// # Examples
    ///
    /// ```rust, compile_fail
    /// let result = websocket
    ///   .write_message(protocol::Message::Text(message.to_string()))
    ///   .await;
    /// if result.is_err() {
    ///     output.send(Event::Disconnected).await;
    ///     break;
    /// }
    /// ```
    ///
    /// # Panics
    ///
    /// This function may panic if the WebSocket frame processing fails.
    ///
    ///
    fn write_message(&mut self, message: Message) -> impl Future<Output = VetisResult<()>>;
}

/// Trait for WebSockets
pub trait WebSocketExt {
    /// Close connection
    fn send_close(&mut self, code: u16, reason: &str) -> impl Future<Output = VetisResult<()>>;
    /// Send a text message
    fn send_text(&mut self, message: &str) -> impl Future<Output = VetisResult<()>>;
    /// Send binary content
    fn send_binary(&mut self, message: &[u8]) -> impl Future<Output = VetisResult<()>>;
    /// Send ping message
    fn send_ping(&mut self, message: &[u8]) -> impl Future<Output = VetisResult<()>>;
    /// Send pong message
    fn send_pong(&mut self, message: &[u8]) -> impl Future<Output = VetisResult<()>>;
}
