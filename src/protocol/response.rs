use bytes::BytesMut;
use color_eyre::eyre::Result;

use super::{errors::DecodePacketError, ProtocolMode, StationPacket};

#[derive(Debug, PartialEq, Eq)]
pub struct Response {
    pub is_nak: bool,
    pub protocol_mode: ProtocolMode,
    pub packet_id: u8,
    pub data: BytesMut,
}

impl Response {
    pub fn new(protocol_mode: ProtocolMode, packet_id: u8, data: BytesMut) -> Self {
        return Self {
            is_nak: false,
            protocol_mode,
            packet_id,
            data,
        };
    }

    pub fn new_nak() -> Self {
        return Self {
            is_nak: true,
            protocol_mode: ProtocolMode::Extended,
            packet_id: 0x00,
            data: BytesMut::new(),
        };
    }
}

impl Response {
    pub fn into_packet<P: StationPacket>(&self) -> Result<P, DecodePacketError> {
        if self.is_nak {
            return Err(DecodePacketError::InputIsNak);
        }

        if self.packet_id != P::PACKET_ID {
            return Err(DecodePacketError::WrongPacketId);
        }

        P::decode(&self.data)
    }
}
