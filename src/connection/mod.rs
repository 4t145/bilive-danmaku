#[derive(Debug)]
pub enum WsConnectError {
    #[cfg(feature = "rt_tokio")]
    WsError(tokio_tungstenite::tungstenite::Error),
    #[cfg(feature = "rt_wasm")]
    WsError(gloo_net::websocket::WebSocketError),
    #[cfg(feature = "rt_wasm")]
    JsError(gloo_utils::errors::JsError),
    UnexpecedEnd,
    AuthFailed,
}

impl std::fmt::Display for WsConnectError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use WsConnectError::*;
        match self {
            #[cfg(feature = "rt_tokio")]
            WsError(e) => write!(f, "WebSocket错误：{}", e),
            #[cfg(feature = "rt_wasm")]
            JsError(e) => write!(f, "javascript 错误{}", e),
            #[cfg(feature = "rt_wasm")]
            WsError(e) => write!(f, "WebSocket错误：{}", e),
            UnexpecedEnd => write!(f, "连接意外关闭"),
            AuthFailed => write!(f, "鉴权失败"),
        }
    }
}

impl std::error::Error for WsConnectError {}

#[derive(Debug, Clone)]
pub enum EventStreamError {
    ConnectionClosed,
    WsError(String),
}

impl std::fmt::Display for EventStreamError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use EventStreamError::*;
        match self {
            ConnectionClosed => write!(f, "连接已关闭"),
            WsError(e) => write!(f, "WebSocket错误：{}", e),
        }
    }
}

impl std::error::Error for EventStreamError {}
// #[async_trait]
// pub trait Connector: Stream<Item = Result<Event, EventStreamError>> + StreamExt
// where
//     Self: Sized,
// {
//     /// 连接
//     // async fn connect(url: String, auth: Auth) -> Result<Self, WsConnectError>;
//     /// abort所有任务
//     fn abort(self);
// }

#[derive(Debug, Clone)]
pub struct LoginInfo {
    // SESSDATA
    sessdata: String,
}

impl LoginInfo {
    pub fn new(sessdata: String) -> Self {
        Self { sessdata }
    }
    pub fn inject(&self, mut request: Request) -> Request {
        let cookie = HeaderValue::from_str(&format!("SESSDATA={}", self.sessdata)).expect("invalid sessdata");
        request.headers_mut().append(COOKIE, cookie);
        request
    }
}

#[cfg(feature = "rt_tokio")]
mod tokio_connection;
use reqwest::header::{COOKIE, HeaderValue};
#[cfg(feature = "rt_tokio")]
pub use tokio_connection::TokioConnection as Connection;

#[cfg(feature = "rt_wasm")]
mod wasm_connection;
use tokio_tungstenite::tungstenite::handshake::client::Request;
#[cfg(feature = "rt_wasm")]
pub use wasm_connection::WasmConnection as Connection;
