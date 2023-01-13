use async_trait::async_trait;
use futures_util::{Stream, StreamExt};

use crate::{
    connect::{Auth},
    event::EventData,
};

pub enum WsConnectError {
    FailToConnect,
    FailToSendAuth,
    FailToAuth,
}

#[async_trait]
pub trait WsConnection
where
    Self: Sized,
{
    async fn connect(url: String, auth: Auth) -> Result<Self, WsConnectError>;
    fn abort(self) -> (String, Auth);
}

pub trait Unpacker: Stream<Item = EventData> + StreamExt {
    type Connection: WsConnection;
    fn unpack(connection: Self::Connection) -> Self;
    fn abort(self);
}

#[cfg(feature = "rt_tokio")]
pub mod ws_tokio {
    use async_trait::async_trait;
    use futures_util::{SinkExt, Stream, StreamExt};
    // use tungstenite;
    use crate::{
        connect::{Auth, Operation, RawPacket},
        event::EventData,
        ws::WsConnectError,
    };
    use tokio::sync;
    use tokio_tungstenite as tokio_ws2;
    use tokio_ws2::tungstenite as ws2;

    use super::{Unpacker, WsConnection};

    type WsStream = tokio_ws2::WebSocketStream<tokio_ws2::MaybeTlsStream<tokio::net::TcpStream>>;

    #[derive(Debug)]
    pub struct TokioConnection {
        url: String,
        auth: Auth,
        rx: sync::mpsc::Receiver<RawPacket>,
        hb_handle: tokio::task::JoinHandle<()>,
        rx_handle: tokio::task::JoinHandle<()>,
    }

    #[async_trait]
    impl WsConnection for TokioConnection {
        // type Connection = impl futures_util::Future<Output = Result<WsConnector, WsConnectError>>;
        async fn connect(url: String, auth: Auth) -> Result<Self, WsConnectError> {
            use ws2::Message::*;
            let conn_result = tokio_ws2::connect_async(url.clone()).await;
            let mut ws_stream = match conn_result {
                Ok((stream, resp)) => stream,
                Err(e) => {
                    return Err(WsConnectError::FailToConnect);
                }
            };

            let authpack_bin = RawPacket::build(Operation::Auth, auth.clone().ser()).ser();
            ws_stream
                .send(Binary(authpack_bin))
                .await
                .map_err(|_| WsConnectError::FailToSendAuth)?;
            let _auth_reply = match ws_stream.next().await {
                Some(Ok(Binary(auth_reply_bin))) => RawPacket::from_buffer(&auth_reply_bin),
                other @ _ => {
                    // exception.send(Exception::FailToAuth).await.unwrap();
                    return Err(WsConnectError::FailToAuth);
                }
            };
            let (mut tx, mut rx) = ws_stream.split();
            // hb thread
            let hb = async move {
                tx.send(ws2::Message::Binary(RawPacket::heartbeat().ser()))
                    .await;
                tokio::time::sleep(tokio::time::Duration::from_secs(30));
            };
            let (packet_tx, packet_rx) = sync::mpsc::channel::<RawPacket>(256);
            let recv = async move {
                while let Some(Ok(msg)) = rx.next().await {
                    match msg {
                        Binary(bin) => {
                            let packet = RawPacket::from_buffer(&bin);
                            packet_tx.send(packet).await;
                            // ws_stream.write_message(packet.ser()).unwrap_or_default();
                        }
                        Close(f) => {
                            break;
                        }
                        Ping(_) | Pong(_) => {}
                        msg @ _ => {
                            //
                        }
                    }
                }
            };
            return Ok(TokioConnection {
                url,
                auth,
                rx: packet_rx,
                hb_handle: tokio::spawn(hb),
                rx_handle: tokio::spawn(recv),
            });
        }

        fn abort(self) -> (String, crate::connect::Auth) {
            self.hb_handle.abort();
            self.rx_handle.abort();
            return (self.url, self.auth);
        }
    }
// 动物化的后现代
    pub struct TokioUnpacker {
        rx: sync::mpsc::Receiver<EventData>,
        handle: tokio::task::JoinHandle<TokioConnection>,
    }

    impl Stream for TokioUnpacker {
        type Item = EventData;

        fn poll_next(
            self: std::pin::Pin<&mut Self>,
            cx: &mut std::task::Context<'_>,
        ) -> std::task::Poll<Option<Self::Item>> {
            self.rx.poll_recv(cx)
        }
    }
    
    impl Unpacker for TokioUnpacker {
        type Connection = TokioConnection;
        fn unpack(connection: Self::Connection) -> Self {
            let (tx, rx) = sync::mpsc::channel::<EventData>(512);
            let TokioConnection {
                url,
                auth,
                rx: packet_rx,
                hb_handle,
                rx_handle,
            } = connection;
            let tx_clone = tx.clone();
            let task = async move {
                while let Some(packet) = packet_rx.recv().await {
                    for data in packet.clone().get_datas() {
                        match data.to_event() {
                            Ok(Some(e)) => {
                                tx.send(e).await;
                            },
                            Err(_) => {

                            },
                            _ => {}
                        }
                    }
                }
                return connection
            };
            return Self {
                rx,
                handle: tokio::spawn(task),
            };
        }

        fn abort(self) {
            self.handle.abort()
        }
    }
}
