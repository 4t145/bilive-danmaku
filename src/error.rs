use crate::cmd::CmdDeserError;
use crate::connection::{EventStreamError, WsConnectError};
#[derive(Debug)]
pub enum Error {
    CmdDeserialize(CmdDeserError),
    BiliClientError(bilibili_client::reqwest_client::ClientError),
    EventStream(EventStreamError),
    WsConnect(WsConnectError),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::CmdDeserialize(e) => f.write_fmt(format_args!("命令解析错误：{e}")),
            Error::BiliClientError(e) => f.write_fmt(format_args!("bilibili 客户端错误： {e:?}")),
            Error::EventStream(e) => f.write_fmt(format_args!("事件流错误：{e}")),
            Error::WsConnect(e) => f.write_fmt(format_args!("建立websocket连接错误: {e}")),
        }
    }
}
