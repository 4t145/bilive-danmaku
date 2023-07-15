use crate::cmd::CmdDeserError;
use crate::connection::{EventStreamError, WsConnectError};
use crate::InitError;

#[derive(Debug)]
pub enum Error {
    CmdDeserialize(CmdDeserError),
    Init(InitError),
    EventStream(EventStreamError),
    WsConnect(WsConnectError),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::CmdDeserialize(e) => f.write_fmt(format_args!("命令解析错误：{e}")),
            Error::Init(e) => f.write_fmt(format_args!("连接初始化错误：{e}")),
            Error::EventStream(e) => f.write_fmt(format_args!("事件流错误：{e}")),
            Error::WsConnect(e) => f.write_fmt(format_args!("建立websocket连接错误: {e}")),
        }
    }
}
