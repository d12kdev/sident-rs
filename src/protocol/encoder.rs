use bytes::{BufMut, BytesMut};

use super::{
    calculate_crc,
    consts::{ETX, STX},
    HostPacket,
};

#[derive(Debug)]
pub struct Encoder {
    ensure_detection: bool,
}

impl Encoder {
    pub fn new(ensure_detection: bool) -> Self {
        Self { ensure_detection }
    }

    pub fn set_ensure_detection(&mut self, ensure_detection: bool) {
        self.ensure_detection = ensure_detection;
    }

    pub fn is_extended_instruction(id: &u8) -> bool {
        let id = *id;

        match id {
            0xC4 => return false,
            id => return id >= 0x80,
        }
    }

    pub fn encode<P: HostPacket>(&self, packet: P) -> BytesMut {
        let mut buffer = BytesMut::new();

        if self.ensure_detection {
            buffer.put_u8(0xFF);
            buffer.put_u8(STX);
        }

        buffer.put_u8(STX);
        buffer.put_u8(P::PACKET_ID);

        let mut data = BytesMut::new();
        packet.write(&mut data);

        if Self::is_extended_instruction(&P::PACKET_ID) {
            buffer.put_u8(data.len() as u8);
            buffer.extend_from_slice(&data);
            buffer.put_u16_le(calculate_crc(&data));
        } else {
            buffer.extend_from_slice(&data);
        }

        buffer.put_u8(ETX);

        return buffer;
    }
}