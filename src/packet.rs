use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use std::{
    fmt::Display,
    io::{Cursor, Read, Write},
};
// enable these functions after `split_array` is stable
/* fn write_u32_be(writer: &mut [u8], val: u32) -> &mut [u8] {
    let (write, writer) = writer.split_array_mut::<4>();
    *write = val.to_be_bytes();
    writer
}

fn write_u16_be(writer: &mut [u8], val: u16) -> &mut [u8] {
    let (write, writer) = writer.split_array_mut::<2>();
    *write = val.to_be_bytes();
    writer
} */

/* fn read_u32_be(buffer: &[u8]) -> (u32, &[u8]) {
    let (read, tail) = buffer.split_array::<4>();
    (u32::from_be_bytes(*read), tail)
}

fn read_u16_be(buffer: &[u8]) -> (u16, &[u8]) {
    let (read, tail) = buffer.split_array_ref::<2>();
    (u16::from_be_bytes(*read), tail)
} */

#[derive(Debug, Clone)]
pub enum Data {
    Json(serde_json::Value),
    Popularity(u32),
    Deflate(String),
}

pub enum EventParseError {
    CmdDeserError(CmdDeserError),
    DeflateMessage,
}

impl Display for EventParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EventParseError::CmdDeserError(e) => write!(f, "CmdDeserError: {}", e),
            EventParseError::DeflateMessage => write!(f, "DeflateMessage"),
        }
    }
}

impl Data {
    pub fn into_event_data(self) -> Result<Option<EventData>, EventParseError> {
        let data = match self {
            Data::Json(json_val) => match crate::cmd::Cmd::deser(json_val) {
                Ok(cmd) => cmd.into_event(),
                Err(e) => return Err(EventParseError::CmdDeserError(e)),
            },
            Data::Popularity(popularity) => Some(PopularityUpdateEvent { popularity }.into()),
            Data::Deflate(_) => return Err(EventParseError::DeflateMessage),
        };
        Ok(data)
    }
}

#[derive(Debug, Clone)]
struct RawPacketHead {
    size: u32,
    header_size: u16,
    proto_code: u16,
    opcode: u32,
    sequence: u32,
}

#[repr(transparent)]
#[derive(Debug, Clone)]
struct RawPacketData<'p>(&'p [u8]);

#[derive(Debug, Clone)]
pub struct RawPacket<'p> {
    head: RawPacketHead,
    data: RawPacketData<'p>,
}

impl<'p> RawPacket<'p> {
    pub fn heartbeat() -> Self {
        RawPacket {
            head: RawPacketHead {
                size: 31,
                header_size: 16,
                proto_code: 1,
                opcode: 2,
                sequence: 1,
            },
            data: RawPacketData(b"[object Object]"),
        }
    }

    pub(crate) fn from_buffer(buffer: &'p [u8]) -> Self {
        const READ_FAIL_ERR: &str = "read raw packet error";
        let mut cursor = Cursor::new(buffer);
        let size = cursor.read_u32::<BigEndian>().expect(READ_FAIL_ERR);
        let header_size = cursor.read_u16::<BigEndian>().expect(READ_FAIL_ERR);
        let version = cursor.read_u16::<BigEndian>().expect(READ_FAIL_ERR);
        let opcode = cursor.read_u32::<BigEndian>().expect(READ_FAIL_ERR);
        let sequence = cursor.read_u32::<BigEndian>().expect(READ_FAIL_ERR);
        let head = RawPacketHead {
            size,
            header_size,
            proto_code: version,
            opcode,
            sequence,
        };
        let pos = cursor.position();
        let data = RawPacketData(&buffer[(pos as usize)..]);
        RawPacket { head, data }
    }

    fn from_buffers(buffer: &'p [u8]) -> Vec<Self> {
        const READ_FAIL_ERR: &str = "read raw packet error";
        let mut packets = vec![];
        let mut cursor = Cursor::new(buffer);
        loop {
            let size = cursor.read_u32::<BigEndian>().expect(READ_FAIL_ERR);
            let header_size = cursor.read_u16::<BigEndian>().expect(READ_FAIL_ERR);
            let version = cursor.read_u16::<BigEndian>().expect(READ_FAIL_ERR);
            let opcode = cursor.read_u32::<BigEndian>().expect(READ_FAIL_ERR);
            let sequence = cursor.read_u32::<BigEndian>().expect(READ_FAIL_ERR);
            let head = RawPacketHead {
                size,
                header_size,
                proto_code: version,
                opcode,
                sequence,
            };
            let pos = cursor.position();
            let body_size = (size as usize) - (header_size as usize);
            let data = RawPacketData(&buffer[(pos as usize)..(pos as usize) + body_size]);
            packets.push(RawPacket { head, data });
            cursor.set_position(pos + body_size as u64);
            if cursor.position() >= buffer.len() as u64 {
                break;
            }
        }
        packets
    }

    pub fn build(op: Operation, data: &'p [u8]) -> Self {
        let header_size = 16_u16;
        let size = (16 + data.len()) as u32;
        let opcode = op as u32;
        Self {
            head: RawPacketHead {
                size,
                header_size,
                proto_code: 1,
                opcode,
                sequence: 1,
            },
            data: RawPacketData(data),
        }
    }

    pub fn ser(self) -> Vec<u8> {
        const READ_FAIL_ERR: &str = "write raw packet error";
        const HEAD_SIZE: usize = 16;
        let head = self.head;
        let data = self.data.0;
        let mut buffer = Vec::<u8>::with_capacity(128 + data.len());
        buffer.resize(data.len() + HEAD_SIZE, 0);
        let mut writer: &mut [u8] = &mut buffer;
        writer
            .write_u32::<BigEndian>(head.size)
            .expect(READ_FAIL_ERR);
        writer
            .write_u16::<BigEndian>(head.header_size)
            .expect(READ_FAIL_ERR);
        writer
            .write_u16::<BigEndian>(head.proto_code)
            .expect(READ_FAIL_ERR);
        writer
            .write_u32::<BigEndian>(head.opcode)
            .expect(READ_FAIL_ERR);
        writer
            .write_u32::<BigEndian>(head.sequence)
            .expect(READ_FAIL_ERR);
        // writer = write_u32_be(writer, head.size);
        // writer = write_u16_be(writer, head.header_size);
        // writer = write_u16_be(writer, head.proto_code);
        // writer = write_u32_be(writer, head.opcode);
        // writer = write_u32_be(writer, head.sequence);
        writer.write_all(data).expect(READ_FAIL_ERR);
        buffer
    }

    pub fn get_datas(self) -> Vec<Data> {
        match self.head.proto_code {
            // raw json
            0 => {
                if let Ok(data_json) = serde_json::from_slice::<serde_json::Value>(self.data.0) {
                    vec![Data::Json(data_json)]
                } else {
                    // println!("cannot deser {}", String::from_utf8(self.data.0).unwrap() );
                    vec![]
                }
            }
            1 => {
                let (bytes, _) = self.data.0.split_at(4);
                let popularity = u32::from_be_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]);
                vec![Data::Popularity(popularity)]
            }
            2 => {
                #[cfg(feature = "deflate")]
                {
                    let deflated = deflate::deflate_bytes(&self.data.0);
                    let utf8 = String::from_utf8(deflated).unwrap();
                    return vec![Data::Deflate(utf8)];
                }
                #[cfg(not(feature = "deflate"))]
                vec![Data::Deflate("".to_string())]
            }
            3 => {
                let read_stream = Cursor::new(self.data.0);
                let mut input = brotli::Decompressor::new(read_stream, 4096);
                let mut buffer = Vec::new();
                match input.read_to_end(&mut buffer) {
                    Ok(_size) => {
                        let unpacked = RawPacket::from_buffers(&buffer);
                        let mut packets = vec![];
                        for p in unpacked {
                            for sub_p in p.get_datas() {
                                packets.push(sub_p)
                            }
                        }
                        packets
                    }
                    Err(e) => {
                        log::error!("读取数据包解压结果错误：{e}");
                        vec![]
                    }
                }
            }
            _ => {
                log::warn!("不支持的操作码：{}", self.head.proto_code);
                vec![]
            } //
        }
    }
}

#[allow(dead_code)]
#[derive(Debug)]
pub enum Operation {
    Handshake,
    HandshakeReply,
    Heartbeat,
    HeartbeatReply,
    SendMsg,
    SendMsgReply,
    DisconnectReply,
    Auth,
    AuthReply,
    ProtoReady,
    ProtoFinish,
    ChangeRoom,
    ChangeRoomReply,
    Register,
    RegisterReply,
    Unregister,
    UnregisterReply,
}

use serde::Serialize;
const PLATFORM_WEB: &str = "web";
use crate::{
    cmd::CmdDeserError,
    event::{EventData, PopularityUpdateEvent},
};
#[derive(Debug, Clone, Serialize)]
pub struct Auth {
    pub uid: u64,
    pub roomid: u64,
    pub protover: i32,
    pub platform: &'static str,
    pub r#type: i32,
    pub key: Option<String>,
}

impl Auth {
    pub fn new(uid: u64, roomid: u64, key: Option<String>) -> Self {
        Self {
            uid,
            roomid,
            protover: 3,
            platform: PLATFORM_WEB,
            r#type: 2,
            key,
        }
    }

    pub fn ser(self) -> Vec<u8> {
        let jsval = serde_json::json!(self);
        jsval.to_string().as_bytes().to_owned()
    }
}
