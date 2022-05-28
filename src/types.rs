
#[cfg(feature = "connect")]
use tokio_tungstenite as tokio_ws2;
// use tokio_tungstenite::tungstenite as ws2;
// use futures_util::stream::{SplitSink, SplitStream};
#[cfg(feature = "connect")]
use tokio_ws2::{MaybeTlsStream, WebSocketStream};
#[cfg(feature = "connect")]
use tokio::net::TcpStream;
#[cfg(feature = "connect")]
pub(crate) type WsStream = WebSocketStream<MaybeTlsStream<TcpStream>>;

// pub type WsTx = SplitSink<WsStream, ws2::Message>;
// pub type WsRx = SplitStream<WsStream>;