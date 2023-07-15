use super::*;
use futures_util::{stream::SplitStream, SinkExt, Stream, StreamExt};
use std::collections::VecDeque;
// use tungstenite;
use crate::{
    connection::WsConnectError,
    event::Event,
    packet::{Auth, Operation, RawPacket},
};
use tokio_tungstenite as tokio_ws2;
use tokio_ws2::tungstenite as ws2;
type WsStream = tokio_ws2::WebSocketStream<tokio_ws2::MaybeTlsStream<tokio::net::TcpStream>>;
type WsRx = SplitStream<WsStream>;

pub struct TokioConnection {
    ws_rx: WsRx,
    hb_handle: tokio::task::JoinHandle<()>,
    buffer: VecDeque<Result<Event, EventStreamError>>, // rx_handle: tokio::task::JoinHandle<()>,
}

impl Stream for TokioConnection {
    type Item = Result<Event, EventStreamError>;

    fn poll_next(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Option<Self::Item>> {
        use std::task::Poll::*;
        use ws2::Message::*;
        use EventStreamError::*;
        if let Some(event) = self.buffer.pop_front() {
            return Ready(Some(event));
        }
        // 读取新序列
        match self.ws_rx.poll_next_unpin(cx) {
            Ready(Some(Ok(Binary(bin)))) => {
                let packet = RawPacket::from_buffer(&bin);
                for data in packet.get_datas() {
                    match data.into_event() {
                        Ok(Some(event)) => self.buffer.push_back(Ok(event)),
                        Ok(None) => {}
                        Err(e) => {
                            log::warn!("解析数据包失败：{}", e);
                        }
                    }
                }
                self.poll_next(cx)
            }
            Ready(Some(Ok(Close(_)))) => Ready(Some(Err(ConnectionClosed))),
            // 这不太可能发生，可能要标记一下
            Ready(Some(Ok(_))) => self.poll_next(cx),
            // 错误
            Ready(Some(Err(e))) => Ready(Some(Err(WsError(e.to_string())))),
            // 接受到None
            Ready(None) => Ready(None),
            Pending => Pending,
        }
    }
}

impl From<ws2::Error> for WsConnectError {
    fn from(val: ws2::Error) -> Self {
        WsConnectError::WsError(val)
    }
}

impl TokioConnection {
    pub async fn connect(url: String, auth: Auth) -> Result<Self, WsConnectError> {
        use ws2::Message::*;
        let (mut ws_stream, _resp) = tokio_ws2::connect_async(url).await?;
        let authpack_bin = RawPacket::build(Operation::Auth, &auth.ser()).ser();
        ws_stream.send(Binary(authpack_bin)).await?;
        let resp = ws_stream.next().await.ok_or_else(|| {
            log::error!("ws stream encounter unexpected end");
            WsConnectError::UnexpecedEnd
        })??;
        match resp {
            Binary(auth_reply_bin) => {
                log::debug!("auth reply: {:?}", RawPacket::from_buffer(&auth_reply_bin));
            }
            _other => {
                log::error!("auth reply is not a binary: {:?}", _other);
                return Err(WsConnectError::AuthFailed);
            }
        }
        let (mut tx, rx) = ws_stream.split();
        // hb task
        let hb = async move {
            use tokio::time::*;
            // 30s 发送一次
            let mut interval = interval(Duration::from_secs(30));
            loop {
                interval.tick().await;
                tx.send(ws2::Message::Binary(RawPacket::heartbeat().ser()))
                    .await
                    .expect("hb send error");
            }
        };
        Ok(TokioConnection {
            ws_rx: rx,
            hb_handle: tokio::spawn(hb),
            buffer: VecDeque::with_capacity(256),
        })
    }

    pub fn abort(self) {
        self.hb_handle.abort();
    }
}
// 动物化的后现代
