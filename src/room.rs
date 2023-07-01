// #[allow(dead_code)]
use serde::Deserialize;

use crate::{connection::*, packet::*};

#[derive(Debug, Clone)]
pub struct Connector {
    pub roomid: u64,
    pub key: String,
    pub host_index: usize,
    pub host_list: Vec<Host>,
}

#[derive(Debug)]
pub enum InitError {
    ParseError,
    HttpError,
    DeserError(serde_json::Error),
}

#[derive(Debug)]
pub enum Exception {
    FailToAuth,
    WsSendError(String),
    WsDisconnected(String),
}
impl Connector {
    pub async fn init(mut roomid: u64) -> Result<Self, InitError> {
        let room_info_url = format!(
            "https://api.live.bilibili.com/xlive/web-room/v2/index/getRoomPlayInfo?room_id={}",
            roomid
        );
        match surf::get(room_info_url).await {
            Ok(mut resp) => {
                if resp.status().is_success() {
                    if let Ok(body) = resp.body_string().await {
                        let response_json_body: RoomPlayInfo =
                            serde_json::from_str(body.as_str()).map_err(InitError::DeserError)?;
                        if let Some(data) = response_json_body.data {
                            roomid = data.room_id;
                        }
                    } else {
                        return Err(InitError::ParseError);
                    }
                } else {
                    return Err(InitError::HttpError);
                }
            }
            Err(_) => return Err(InitError::HttpError),
        }
        let url = format!(
            "https://api.live.bilibili.com/xlive/web-room/v1/index/getDanmuInfo?id={}&type=0",
            roomid
        );
        match surf::get(url).await {
            Ok(mut resp) => {
                if resp.status().is_success() {
                    if let Ok(body) = resp.body_string().await {
                        let response_json_body: Response =
                            serde_json::from_str(body.as_str()).map_err(InitError::DeserError)?;
                        let disconnected = Connector {
                            host_index: 0,
                            roomid,
                            key: response_json_body.data.token,
                            host_list: response_json_body.data.host_list,
                        };
                        Ok(disconnected)
                    } else {
                        Err(InitError::ParseError)
                    }
                } else {
                    Err(InitError::HttpError)
                }
            }
            Err(_) => Err(InitError::HttpError),
        }
    }

    pub fn use_host(&mut self, index: usize) -> Result<&'_ str, usize> {
        if self.host_list.len() > index {
            self.host_index = index;
            Ok(&self.host_list[index].host)
        } else {
            Err(self.host_list.len())
        }
    }

    pub async fn connect(&self) -> Result<Connection, ConnectError> {
        if self.host_list.is_empty() {
            return Err(ConnectError::HostListIsEmpty);
        }
        let url = self.host_list[self.host_index].wss();
        let roomid = self.roomid;
        let backup = self.clone();
        let auth = Auth::new(0, roomid, Some(backup.key.clone()));
        let stream = Connection::connect(url, auth)
            .await
            .map_err(|_| ConnectError::HandshakeError)?;
        Ok(stream)
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
    data: Option<RoomPlayInfoData>,
}

#[derive(Debug, Deserialize)]
struct Response {
    // code: i32,
    // message: String,
    // ttl: i32,
    data: ResponseData,
}
#[derive(Debug, Deserialize)]

struct ResponseData {
    // max_delay: i32,
    token: String,
    host_list: Vec<Host>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Host {
    pub host: String,
    pub wss_port: u16,
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
    HandshakeError,
    WsError(String),
}
