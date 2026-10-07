//! WebSockets module
use crate::errors::WebSocketError;
use base64::{engine::general_purpose::STANDARD, Engine};
use deboa::{
    request::{DeboaRequest, DeboaRequestBuilder},
    url::IntoUrl,
};
use http::{header, Method};
use pin_project_lite::pin_project;
use std::{
    future::Future,
    sync::{Arc, Mutex},
};

pub mod errors;

/// Smol runtime support
#[cfg(feature = "runtime-smol")]
pub mod smol;
/// Tokio runtime support
#[cfg(feature = "runtime-tokio")]
pub mod tokio;

/// Result alias
pub type Result<T> = std::result::Result<T, WebSocketError>;

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
    /// A binary message
    Binary(Vec<u8>),
    /// Close message
    Close(u16, String),
    /// Ping message
    Ping(Vec<u8>),
    /// Pong reply message
    Pong(Vec<u8>),
}

/// Trait for building websocket requests
pub trait WebsocketRequestBuilder {
    /// Creates a websocket request
    ///
    /// # Arguments
    ///
    /// * `url` - The URL to connect to
    ///
    /// # Returns
    ///
    /// A Result containing the DeboaRequestBuilder
    ///
    /// # Example
    ///
    /// ``` compile_fail
    /// use deboa::{Client, Result, request::{IntoUrl, DeboaRequestBuilder}};
    /// use deboa_ws::WebsocketRequestBuilder;
    ///
    /// let mut client = Client::new();
    /// let request = DeboaRequestBuilder::websocket("ws://example.com").unwrap();
    /// let response = request.send_with(&mut client).await.unwrap();
    /// let ws = response.into_websocket().unwrap();
    /// loop {
    ///     if let Ok(Some(message)) = ws.read_message().await {
    ///         println!("message: {}", message);
    ///     }
    /// }
    /// ```
    fn websocket<T: IntoUrl>(url: T) -> deboa::Result<DeboaRequestBuilder>;
}

impl WebsocketRequestBuilder for DeboaRequestBuilder {
    fn websocket<T: IntoUrl>(url: T) -> deboa::Result<DeboaRequestBuilder> {
        let rnd: [u8; 16] = rand::random();
        let key = STANDARD.encode(rnd);
        DeboaRequest::at(url, Method::GET)?
            .header(header::UPGRADE, "websocket")?
            .header(header::CONNECTION, "Upgrade")?
            .header(header::SEC_WEBSOCKET_KEY, &key)?
            .header(header::SEC_WEBSOCKET_VERSION, "13")
    }
}

/// Trait for converting a DeboaResponse into a WebSocket
pub trait IntoWebSocket {
    type UpgradedIo;
    /// Converts a DeboaResponse into a WebSocket
    ///
    /// # Arguments
    ///
    /// * `self` - The DeboaResponse to convert
    ///
    /// # Returns
    ///
    /// A Result containing the WebSocket
    ///
    /// # Example
    ///
    /// ``` compile_fail
    /// use deboa::{Client, Result, request::{IntoUrl, DeboaRequestBuilder}};
    /// use deboa_ws::WebsocketRequestBuilder;
    ///
    /// let mut client = Client::new();
    /// let builder = DeboaRequestBuilder::websocket("ws://example.com").unwrap();
    /// let response = builder
    ///     .send_with(&mut client)
    ///     .await
    ///     .unwrap();
    /// let websocket = response.into_websocket().unwrap();
    ///
    /// loop {
    ///     if let Ok(Some(message)) = websocket.read_message().await {
    ///         println!("message: {}", message);
    ///     }
    /// }
    /// ```
    fn into_websocket(self) -> impl Future<Output = deboa::Result<WebSocket<Self::UpgradedIo>>>;
}

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
    fn read_message(&mut self) -> impl Future<Output = Result<Option<Message>>>;
}

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
    fn write_message(&mut self, message: Message) -> impl Future<Output = Result<()>>;
}

/// Trait for WebSockets
pub trait WebSocketExt {
    /// Close connection
    fn send_close(&mut self, code: u16, reason: &str) -> impl Future<Output = Result<()>>;
    /// Send a text message
    fn send_text(&mut self, message: &str) -> impl Future<Output = Result<()>>;
    /// Send binary content
    fn send_binary(&mut self, message: &[u8]) -> impl Future<Output = Result<()>>;
    /// Send ping message
    fn send_ping(&mut self, message: &[u8]) -> impl Future<Output = Result<()>>;
    /// Send pong message
    fn send_pong(&mut self, message: &[u8]) -> impl Future<Output = Result<()>>;
}

pin_project! {
    /// WebSocket struct
    pub struct WebSocket<T>
    {
        #[pin]
        inner: T,
    }
}

impl<T> WebSocket<T> {
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
    pub fn new(inner: T) -> Self {
        Self { inner: inner }
    }

    pub fn split(self) -> (WebSocketReader<T>, WebSocketWriter<T>)
    where
        Self: WebSocketRead + WebSocketWrite,
    {
        let inner_arc = Arc::new(Mutex::new(self));
        (WebSocketReader(inner_arc.clone()), WebSocketWriter(inner_arc))
    }
}

pub struct WebSocketReader<T>(pub(crate) Arc<Mutex<WebSocket<T>>>);

impl<T> WebSocketRead for WebSocketReader<T>
where
    T: WebSocketRead,
{
    async fn read_message(&mut self) -> Result<Option<Message>> {
        let mut guard = self
            .0
            .lock()
            .map_err(|e| WebSocketError::ReceiveMessage { message: e.to_string() })?;
        guard
            .inner
            .read_message()
            .await
    }
}

pub struct WebSocketWriter<T>(pub(crate) Arc<Mutex<WebSocket<T>>>);

impl<T> WebSocketWrite for WebSocketWriter<T>
where
    T: WebSocketWrite,
{
    async fn write_message(&mut self, message: Message) -> Result<()> {
        let mut guard = self
            .0
            .lock()
            .map_err(|e| WebSocketError::ReceiveMessage { message: e.to_string() })?;
        guard
            .inner
            .write_message(message)
            .await
    }
}
