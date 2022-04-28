#![feature(split_array)]

use std::io::Write;
mod operation;
#[derive(Debug)]
struct PacketHead {
    size: i32,
    header_size: i16,
    version: i16,
    opcode: i32,
    sequence: i32,
}

#[repr(transparent)]
#[derive(Debug)]
struct PacketData(Vec<u8>);

#[derive(Debug)]
pub struct Packet {
    head: PacketHead,
    data: PacketData
}

impl Packet {
    pub fn from_buffer(buffer: Vec<u8>) -> Self {

        fn read_i32_be(buffer: &[u8]) -> (i32, &[u8]) {
            let (read, tail) = buffer.split_array_ref::<4>();
            (i32::from_be_bytes(*read), tail)
        }
        
        fn read_i16_be(buffer: &[u8]) -> (i16, &[u8]) {
            let (read, tail) = buffer.split_array_ref::<2>();
            (i16::from_be_bytes(*read), tail)
        }

        let (size, buffer)= read_i32_be(&buffer);
        let (header_size, buffer)= read_i16_be(buffer);
        let (version, buffer)= read_i16_be(buffer);
        let (opcode, buffer)= read_i32_be(buffer);
        let (sequence, buffer)= read_i32_be(buffer);
        let head = PacketHead {
            size,
            header_size,
            version,
            opcode,
            sequence,
        };
    
        let data = PacketData(buffer.to_owned());
    
        Packet {head, data}
    }

    pub fn build(op:Operation, data: Vec<u8>) -> Self {
        let header_size = data.len() as i16;
        let size = (16 + header_size) as i32;
        let opcode = op as i32;
        Self {
            head: PacketHead { 
                size, 
                header_size, 
                version: 1, 
                opcode, 
                sequence:1 
            },
            data: PacketData(data)
        }
    }

    pub fn ser(self) -> Vec<u8> {
        fn write_i32_be(writer: &mut [u8], val: i32) -> &mut [u8] {
            let (write, writer) = writer.split_array_mut::<4>();
            *write = val.to_be_bytes();
            writer
        }
        
        fn write_i16_be(writer: &mut [u8], val: i16) -> &mut [u8] {
            let (write, writer) = writer.split_array_mut::<2>();
            *write = val.to_be_bytes();
            writer
        }

        let head = self.head;
        let data = self.data.0;
        let mut buffer = unsafe {
            let len = 16+data.len();
            let mut v = Vec::<u8>::with_capacity(128+data.len());
            v.set_len(len);
            v
        };

        let mut writer:&mut [u8] = &mut buffer;
        writer = write_i32_be(writer, head.size);
        writer = write_i16_be(writer, head.header_size);
        writer = write_i16_be(writer, head.version);
        writer = write_i32_be(writer, head.opcode);
        writer = write_i32_be(writer, head.sequence);
        writer.write(&data).unwrap();
        buffer
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

impl Operation {
    unsafe fn from_i32(code:i32) -> Self {
        let code = code as u8;
        std::mem::transmute(code)
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn from_buffer_test() {
        let p = Packet::from_buffer(vec![
            0, 0, 0, 26, 0, 16, 0, 1, 0, 0, 0, 8, 0, 0, 0, 1, 123, 34, 99, 111, 100, 101, 34, 58,
            48, 125,
        ]);
        dbg!(p);
    }

    #[test]
    fn pack() {
        let head = PacketHead {
            size: 16+16,
            header_size: 16,
            version: 1,
            opcode: 2,
            sequence: 1,
        };
        let data = PacketData (vec![0x33;16]);
        let p = Packet { head, data };
        let buf = p.ser();
        dbg!(buf);
    }

    #[test]
    fn build() {
        let head = Packet::build(Operation::UnregisterReply, vec![0,0,0,0]);
        dbg!(head);
    }
}
