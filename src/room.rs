
// #[allow(dead_code)]
use serde::{Deserialize};
use tokio_tungstenite as tokio_ws2;
use tokio_tungstenite::tungstenite as ws2;
use futures_util::{StreamExt, SinkExt};

use tokio::{sync::{mpsc, broadcast}, task::JoinHandle};

use crate::connect::*;
#[derive(Debug, Clone)]
pub struct Uninited;
#[derive(Debug, Clone)]
pub struct Disconnected {
    key: String,
    host_list: Vec<Host>,
}
#[derive(Debug)]
pub struct Connected {
    fallback: Disconnected,
    broadcastor: broadcast::Sender<Event>,
    exception_watcher: mpsc::Receiver<Exception>,
    process_handle: JoinHandle<()>,
    conn_handle: RoomConnectionHandle
}

#[derive(Debug)]
pub struct RoomService<S> {
    roomid: u64,
    status: S,
}

impl RoomService<()> {
    pub fn new(roomid: u64) -> RoomService<Uninited> {
        RoomService {
            roomid,
            status: Uninited{},
        }
    }
}

impl RoomService<Uninited> {
    pub async fn init(mut self) -> Result<RoomService<Disconnected>, (Self, ())> {
        let room_info_url = format!("https://api.live.bilibili.com/xlive/web-room/v2/index/getRoomPlayInfo?room_id={}", self.roomid);
        match reqwest::get(room_info_url).await {
            Ok(resp) => {
                if resp.status().is_success() {
                    if let Ok(body) = resp.text().await {
                        let response_json_body:RoomPlayInfo = serde_json::from_str(body.as_str()).unwrap();
                        if let Some(data) = response_json_body.data {
                            self.roomid = data.room_id;
                        }
                    } else {
                        return Err((self, ()))
                    }
                } else {
                    return Err((self, ()))
                }
            }
            Err(_) => {
                return Err((self, ()))
            },
        }
        let url = format!("https://api.live.bilibili.com/xlive/web-room/v1/index/getDanmuInfo?id={}&type=0", self.roomid);
        match reqwest::get(url).await {
            Ok(resp) => {
                if resp.status().is_success() {
                    if let Ok(body) = resp.text().await {
                        let response_json_body:Response = serde_json::from_str(body.as_str()).unwrap();
                        let status = Disconnected {
                            key: response_json_body.data.token,
                            host_list: response_json_body.data.host_list
                        };
                        Ok(RoomService {
                            roomid: self.roomid,
                            status
                        })
                    } else {
                        Err((self, ()))
                    }
                } else {
                    Err((self, ()))
                }
            }
            Err(_) => {
                Err((self, ()))
            },
        }
    }
}

#[derive(Debug)]
pub enum Exception {
    FailToAuth,
    WsDisconnected,
}
impl RoomService<Disconnected> {
    pub async fn connect(self) -> Result<RoomService<Connected>, (Self, ConnectError)> {
        if self.status.host_list.is_empty() {
            return Err((self, ConnectError::HostListIsEmpty));
        }
        let url = self.status.host_list[0].wss();
        let roomid = self.roomid;
        let status = self.status.clone();
        match tokio_ws2::connect_async(url).await {
            Ok((stream, _)) => {
                let (exception_repoter, exception_watcher) = mpsc::channel::<Exception>(4);
                let auth = Auth::new( 0, roomid, Some(status.key.clone()));
                let mut conn = RoomConnection::start(stream, auth, exception_repoter).await
                .map_err(move |_|(RoomService {roomid, status}, ConnectError::FailToStart))?;
                let (broadcastor, _) = broadcast::channel::<Event>(128);
                let process_packet_broadcastor = broadcastor.clone();
                let process_packet = async move {
                    while let Some(packet) = conn.pack_rx.recv().await {
                        for data in packet.clone().get_datas() {
                            match data {
                                Data::Json(json_val) => {
                                    match crate::cmd::Cmd::deser(json_val) {
                                        Ok(cmd) => {
                                            if let Some(evt) = cmd.as_event() {
                                                process_packet_broadcastor
                                                .send(evt)
                                                .unwrap_or_default();
                                            }
                                        }
                                        Err(_e) => {
                                            #[cfg(feature = "debug")]
                                            println!("{}", _e);
                                        }
                                    }
                                },
                                Data::Popularity(popularity) => {
                                    process_packet_broadcastor.send(
                                        Event::PopularityUpdate { popularity }
                                    ).unwrap_or_default();
                                },
                                Data::Deflate(s) => {
                                    println!("deflate 压缩的消息（请报告此bug）: \n{}", s);
                                },
                            }
                        }
                    }
                };
                let process_handle = tokio::spawn(process_packet);
                let status = Connected {
                    fallback: self.status,
                    exception_watcher,
                    broadcastor,
                    conn_handle: conn.handle,
                    process_handle,
                    // exception_flag: exception_notify,
                };
                Ok(RoomService {
                    roomid,
                    status
                })
            }
            Err(e) => {
                Err((RoomService {
                    roomid,
                    status
                }, ConnectError::WsError(e.to_string())))
            }
        }
    }
}

impl RoomService<Connected> {
    pub fn subscribe(&self) -> broadcast::Receiver<Event> {
        self.status.broadcastor.subscribe()
    }

    pub async fn exception(&mut self) -> Option<Exception> {
        self.status.exception_watcher.recv().await
    }

    pub fn close(self) -> RoomService<Disconnected> {
        self.status.process_handle.abort();
        self.status.conn_handle.hb_handle.abort();
        self.status.conn_handle.send_handle.abort();
        self.status.conn_handle.recv_handle.abort();
        RoomService{
            roomid: self.roomid,
            status: self.status.fallback,
        }
    }
}


#[derive(Debug, Deserialize)]
struct RoomPlayInfoData {
    room_id: u64,
}


/// 
/// api url:
/// https://api.live.bilibili.com/xlive/web-room/v2/index/getRoomPlayInfo?room_id=510
#[derive(Debug, Deserialize)]
struct RoomPlayInfo {
    data: Option<RoomPlayInfoData>
}


#[derive(Debug, Deserialize)]
struct Response {
    // code: i32,
    // message: String,
    // ttl: i32,
    data: ResponseData
}
#[derive(Debug, Deserialize)]

struct ResponseData {
    // max_delay: i32,
    token: String,
    host_list: Vec<Host>
}

#[derive(Debug, Deserialize, Clone)]
struct Host {
    host: String,
    wss_port: u16,
}

impl Host {
    fn wss(&self) -> String {
        let host = &self.host;
        let port = self.wss_port;
        format!("wss://{host}:{port}/sub")
    }
}

#[derive(Debug)]
pub enum ConnectError {
    HostListIsEmpty,
    FailToStart,
    WsError(String),
}

use crate::{types::*, event::Event};
#[derive(Debug)]
struct RoomConnection {
    pack_rx: mpsc::Receiver<RawPacket>,
    handle: RoomConnectionHandle
}

#[derive(Debug)]
struct RoomConnectionHandle {
    send_handle: tokio::task::JoinHandle<()>,
    recv_handle: tokio::task::JoinHandle<()>,
    hb_handle: tokio::task::JoinHandle<()>,
}

impl RoomConnection {
    async fn start(ws_stream: WsStream, auth: Auth, exception: mpsc::Sender<Exception>) -> Result<Self, ()> {
        use ws2::Message::*;

        let (mut tx, mut rx) = ws_stream.split();
        let authpack_bin = RawPacket::build(Operation::Auth, auth.ser()).ser();
        tx.send(Binary(authpack_bin)).await.unwrap();
        let _auth_reply = match rx.next().await {
            Some(Ok(Binary(auth_reply_bin))) => RawPacket::from_buffer(&auth_reply_bin),
            other@_ => {
                println!("{:?}", other);
                exception.send(Exception::FailToAuth).await.unwrap();
                return Err(())
            },
        };
        let channel_buffer_size = 64;
        let (pack_outbound_tx, mut pack_outbound_rx) = mpsc::channel::<RawPacket>(channel_buffer_size);
        let (pack_inbound_tx, pack_inbound_rx) = mpsc::channel::<RawPacket>(channel_buffer_size);

        let hb_sender = pack_outbound_tx.clone();


        let hb = async move {
            use tokio::time::{sleep, Duration};
            loop {
                hb_sender.send(RawPacket::heartbeat()).await.unwrap();
                sleep(Duration::from_secs(30)).await;
            }
        };
        
        let repoter = exception.clone();
        let send = async move {
            while let Some(p) = pack_outbound_rx.recv().await {
                let bin= p.ser();
                tx.send(Binary(bin)).await.unwrap_or_default();
            }
            repoter.send(Exception::WsDisconnected).await.unwrap();
        };

        let repoter = exception.clone();
        let recv = async move {
            while let Some(Ok(msg)) = rx.next().await {
                match msg {
                    Binary(bin) => {                        
                        let packet = RawPacket::from_buffer(&bin);
                        pack_inbound_tx.send(packet).await.unwrap_or_default();
                    },
                    Close(f) => {
                        repoter.send(Exception::WsDisconnected).await.unwrap();
                        println!("{:?}",f);
                    },
                    _ => {

                    }
                }
            }
            repoter.send(Exception::WsDisconnected).await.unwrap();
        };

        let send_handle = tokio::spawn(send);
        let recv_handle = tokio::spawn(recv);
        let hb_handle = tokio::spawn(hb);
        Ok(RoomConnection{
            pack_rx: pack_inbound_rx,
            handle: RoomConnectionHandle { send_handle, recv_handle, hb_handle}
        })
    }
}

