#[derive(Debug, Clone)]
pub enum WsConnectError {
    FailToConnect,
    FailToSendAuth,
    FailToAuth,
}

#[derive(Debug, Clone)]
pub enum EventStreamError {
    ConnectionClosed,
    WsError(String),
}

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

#[cfg(feature = "rt_tokio")]
mod tokio_connection;
#[cfg(feature = "rt_tokio")]
pub use tokio_connection::TokioConnection as Connection;

#[cfg(feature = "rt_wasm")]
mod wasm_connection;
#[cfg(feature = "rt_wasm")]
pub use wasm_connection::WasmConnection as Connection;
