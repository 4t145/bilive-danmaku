use crate::{connection::*, packet::*};
use bilibili_client::{
    api::live::{
        danmu_info::RoomPlayInfo,
        room_play_info::{DanmuInfoData, Host},
    },
    reqwest_client::LoginInfo,
};

#[derive(Clone)]
pub struct Connector {
    pub roomid: u64,
    pub uid: u64,
    pub token: String,
    pub host_index: usize,
    pub host_list: Vec<Host>,
    pub login_info: LoginInfo,
    pub client: bilibili_client::reqwest_client::Client,
}

impl Connector {
    pub async fn init(
        mut roomid: u64,
        login_info: LoginInfo,
    ) -> bilibili_client::reqwest_client::ClientResult<Self> {
        let client = bilibili_client::reqwest_client::Client::default();
        client.set_login_info(&login_info);
        let RoomPlayInfo { room_id, uid } = client.get_room_play_info(roomid).await?;
        roomid = room_id;
        let DanmuInfoData { token, host_list } = client.get_danmu_info(room_id).await?;
        let connector = Connector {
            client,
            uid,
            host_index: 0,
            roomid,
            token,
            host_list,
            login_info,
        };
        Ok(connector)
    }

    pub fn set_login_info(&mut self, login_info: LoginInfo) {
        self.login_info = login_info;
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

        for host in &self.host_list {
            let url = host.wss();
            let auth = Auth::new(self.uid, self.roomid, Some(self.token.clone()));
            match Connection::connect(url, auth, self).await {
                Ok(stream) => return Ok(stream),
                Err(e) => log::warn!("connect error: {:?}", e),
            }
        }
        log::error!("connect error: all host failed");
        Err(ConnectError::HandshakeError)
    }
}

#[derive(Debug)]
pub enum ConnectError {
    HostListIsEmpty,
    HandshakeError,
    WsError(String),
}
