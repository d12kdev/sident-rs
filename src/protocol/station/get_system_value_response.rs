use bytes::{Buf, BytesMut};

use crate::protocol::{pckt, StationPacket};

pckt!(GetSystemValueResponse, 0x83);
pub struct GetSystemValueResponse {
    pub station_code: u16,
    pub address: u8,
    pub data: Vec<u8>,
}

impl StationPacket for GetSystemValueResponse {
    fn decode(
        buffer: &[u8],
    ) -> color_eyre::eyre::Result<Self, crate::protocol::errors::DecodePacketError> {
        let mut buffer = BytesMut::from(buffer);
        if buffer.len() < 4 {
            return Err(crate::protocol::errors::DecodePacketError::InputTooShort);
        }

        let station_code = buffer.get_u16_le();
        let address = buffer.get_u8();
        let data = buffer.to_vec();

        return Ok(Self {
            station_code,
            address,
            data,
        });
    }
}
