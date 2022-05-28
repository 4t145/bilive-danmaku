//! # 使用
//! 
//! 
//!```
//!use bilive_danmaku::{RoomService}
//!async fn service() {
//!    let service = RoomService::new(477317922).init().await.unwrap();
//!    let service = service.connect().await.unwrap();
//!    // 这里会获得一个 broadcast::Reciever<Event>
//!    let mut events_rx = service.subscribe();
//!    while let Some(evt) = events_rx.recv().await {
//!        // 处理事件
//!        todo!()
//!    }
//!    let service = service.close();
//!}
//!```

#![allow(dead_code)]
#![feature(split_array)]
#[cfg(feature="connect")]
mod room;
#[cfg(feature="connect")]
pub use crate::room::*;

#[cfg(feature="connect")]
pub(crate) mod cmd;
#[cfg(feature="event")]
pub mod model;
#[cfg(feature="event")]
pub mod event;
pub(crate) mod types;

#[cfg(test)]
mod tests;

#[cfg(feature="connect")]
mod connect {
    use std::io::Write;

    fn write_u32_be(writer: &mut [u8], val: u32) -> &mut [u8] {
        let (write, writer) = writer.split_array_mut::<4>();
        *write = val.to_be_bytes();
        writer
    }
    
    fn write_u16_be(writer: &mut [u8], val: u16) -> &mut [u8] {
        let (write, writer) = writer.split_array_mut::<2>();
        *write = val.to_be_bytes();
        writer
    }
    
    fn read_u32_be(buffer: &[u8]) -> (u32, &[u8]) {
        let (read, tail) = buffer.split_array_ref::<4>();
        (u32::from_be_bytes(*read), tail)
    }
    
    fn read_u16_be(buffer: &[u8]) -> (u16, &[u8]) {
        let (read, tail) = buffer.split_array_ref::<2>();
        (u16::from_be_bytes(*read), tail)
    }
    
    
    
    #[derive(Debug, Clone)]
    pub enum Data {
        Json(serde_json::Value),
        Popularity(u32),
        Deflate(String),
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
    struct RawPacketData(Vec<u8>);
    
    #[derive(Debug, Clone)]
    pub struct RawPacket {
        head: RawPacketHead,
        data: RawPacketData
    }
    
    impl RawPacket {
        pub fn heartbeat() -> Self {
            RawPacket {
                head: RawPacketHead {
                    size: 31,
                    header_size: 16,
                    proto_code: 1,
                    opcode: 2,
                    sequence: 1,
                },
                data: RawPacketData(b"[object Object]".to_vec()),
            }
        }
    
        pub fn from_buffer(buffer: &[u8]) -> Self {
            let (size, buffer)= read_u32_be(&buffer);
            let (header_size, buffer)= read_u16_be(buffer);
            let (version, buffer)= read_u16_be(buffer);
            let (opcode, buffer)= read_u32_be(buffer);
            let (sequence, buffer)= read_u32_be(buffer);
            let head = RawPacketHead {
                size,
                header_size,
                proto_code: version,
                opcode,
                sequence,
            };
        
            let data = RawPacketData(buffer.to_owned());
        
            RawPacket {head, data}
        }
    
        fn from_buffers(buffer: &[u8]) -> Vec<Self> {
            let mut packets = vec![];
            let mut ptr = 0;
            loop {
                let (size, _)= read_u32_be(&buffer[ptr..ptr+4]);
                let size = size as usize;
                packets.push(Self::from_buffer(&buffer[ptr..ptr+size]));
                ptr += size;
                if ptr>=buffer.len() {
                    break;
                }
            }
            return packets;
        }
    
        pub fn build(op:Operation, data: Vec<u8>) -> Self {
            let header_size = 16 as u16;
            let size = (16 + data.len()) as u32;
            let opcode = op as u32;
            Self {
                head: RawPacketHead { 
                    size, 
                    header_size, 
                    proto_code:1, 
                    opcode, 
                    sequence:1 
                },
                data: RawPacketData(data)
            }
        }
    
        pub fn ser(self) -> Vec<u8> {
            let head = self.head;
            let data = self.data.0;
            let mut buffer = unsafe {
                let len = 16+data.len();
                let mut v = Vec::<u8>::with_capacity(128+data.len());
                v.set_len(len);
                v
            };
    
            let mut writer:&mut [u8] = &mut buffer;
            writer = write_u32_be(writer, head.size);
            writer = write_u16_be(writer, head.header_size);
            writer = write_u16_be(writer, head.proto_code);
            writer = write_u32_be(writer, head.opcode);
            writer = write_u32_be(writer, head.sequence);
            writer.write(&data).unwrap();
            buffer
        }
    
        pub fn get_datas(self) -> Vec<Data> {
            match self.head.proto_code {
                // raw json
                0 => {
                    if let Ok(data_json)=serde_json::from_slice::<serde_json::Value>(&self.data.0) {
                        vec!(Data::Json(data_json))
                    } else {
                        // println!("cannot deser {}", String::from_utf8(self.data.0).unwrap() );
                        vec![]
                    }
                }
                1 => {
                    let (bytes, _) = self.data.0.split_array_ref::<4>();
                    let popularity = u32::from_be_bytes(*bytes);
                    vec!(Data::Popularity(popularity))
                }
                2 => {
                    #[cfg(feature = "deflate")] 
                    {
                        let deflated = deflate::deflate_bytes(&self.data.0);
                        let utf8 = String::from_utf8(deflated).unwrap();
                        return vec!(Data::Deflate(utf8));
                    }
                    return vec!(Data::Deflate("".to_string()));
                }
                3 => {
                    use std::io::Read;
                    let read_stream = std::io::Cursor::new(self.data.0) ;
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
                            println!("{}",e);
                            vec![]
                        }     
                    }
                }
                _ => {
                    println!("不支持的操作码：{}", self.head.proto_code);
                    vec![]
                }
                // 
            }
        }
    }
    
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
    
    use serde::{Serialize};
    #[derive(Debug, Serialize)]
    pub struct Auth {
        uid: u64,
        roomid: u64,
        protover: i32,
        platform: &'static str,
        r#type: i32,
        key: Option<String>
    }
    
    impl Auth {
        pub fn new(uid: u64, roomid: u64, key:Option<String>) -> Self {
            Self {
                uid, 
                roomid,
                protover: 3,
                platform: "web",
                r#type: 2,
                key
            }
        }
    
        pub fn ser(self) -> Vec<u8> {
            let jsval = serde_json::json!(self);
            jsval.to_string().as_bytes().to_owned()
        }
    }
}