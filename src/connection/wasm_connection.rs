use super::*;
// use futures_util::{Stream as UtilSr, StreamExt};
use futures::{stream::SplitStream, SinkExt, Stream, StreamExt};
use gloo_net::{self, websocket::futures::WebSocket};
use gloo_timers::future::IntervalStream;
use js_sys::Promise;
use std::collections::VecDeque;


// use tungstenite;
use crate::{
    connector::WsConnectError,
    event::Event,
    packet::{Auth, Operation, RawPacket},
};
use wasm_bindgen_futures::{future_to_promise};
// type WsStream = tokio_ws2::WebSocketStream<tokio_ws2::MaybeTlsStream<tokio::net::TcpStream>>;
type WsRx = SplitStream<WebSocket>;

pub struct WasmConnection {
    ws_rx: WsRx,
    pub hb_handle: Promise,
    buffer: VecDeque<Result<Event, EventStreamError>>, // rx_handle: tokio::task::JoinHandle<()>,
}

impl Stream for WasmConnection {
    type Item = Result<Event, EventStreamError>;

    fn poll_next(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Option<Self::Item>> {
        use gloo_net::websocket::Message::*;
        use std::task::Poll::*;
        use EventStreamError::*;

        if let Some(event) = self.buffer.pop_front() {
            return Ready(Some(event));
        }
        // 读取新序列
        match self.ws_rx.poll_next_unpin(cx) {
            Ready(Some(Ok(Bytes(bin)))) => {
                let packet = RawPacket::from_buffer(&bin);
                for data in packet.get_datas() {
                    match data.to_event() {
                        Ok(Some(event)) => self.buffer.push_back(Ok(event)),
                        _ => {}
                    }
                }
                return self.poll_next(cx);
            }
            // Ready(Some(Ok(Close(_)))) => return Ready(Some(Err(ConnectionClosed))),
            // 这不太可能发生，可能要标记一下
            Ready(Some(Ok(_))) => return self.poll_next(cx),
            // 错误
            Ready(Some(Err(e))) => return Ready(Some(Err(WsError(e.to_string())))),
            // 接受到None
            Ready(None) => {
                return Ready(None);
            }
            Pending => return Pending,
        }
    }
}
impl WasmConnection {
    pub async fn connect(url: String, auth: Auth) -> Result<Self, WsConnectError> {
        use gloo_net::websocket::{ Message::*};
        let conn_result = WebSocket::open(url.as_str());
        let ws_stream = match conn_result {
            Ok(stream) => stream,
            Err(_e) => {
                return Err(WsConnectError::FailToConnect);
            }
        };
    
        let (mut tx, mut rx) = ws_stream.split();
        let authpack_bin = RawPacket::build(Operation::Auth, auth.ser()).ser();
        tx.send(Bytes(authpack_bin))
            .await
            .map_err(|_| WsConnectError::FailToSendAuth)?;
        let _auth_reply = match rx.next().await {
            Some(Ok(Bytes(auth_reply_bin))) => RawPacket::from_buffer(&auth_reply_bin),
            _other @ _ => {
                // exception.send(Exception::FailToAuth).await.unwrap();
                return Err(WsConnectError::FailToAuth);
            }
        };
        // hb task
        let hb = async move {
            // use tokio::time::*;
            // 30s 发送一次
            let mut interval = IntervalStream::new(30000);
            loop {
                interval.next().await;
                tx.send(Bytes(RawPacket::heartbeat().ser())).await.unwrap();
            }
        };
        // let hb = spawn_local();
        return Ok(WasmConnection {
            ws_rx: rx,
            hb_handle: future_to_promise(hb),
            buffer: VecDeque::with_capacity(256),
        });
    }

    pub fn abort(self) {
        // literally do nothing
    }
}
// 动物化的后现代
