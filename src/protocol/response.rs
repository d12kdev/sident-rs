use bytes::BytesMut;

use super::ProtocolMode;

#[derive(Debug, PartialEq, Eq)]
pub struct Response {
    pub protocol_mode: ProtocolMode,
    pub packet_id: u8,
    pub data: BytesMut,
}

impl Response {
    pub fn new(protocol_mode: ProtocolMode, packet_id: u8, data: BytesMut) -> Self {
        return Self {
            protocol_mode,
            packet_id,
            data,
        };
    }
}
