use bytes::{Buf, BytesMut};

use crate::protocol::{pckt, MSMode, StationPacket};

pckt!(SetMSModeResponse, 0xF0);
#[derive(Debug)]
pub struct SetMSModeResponse {
    pub station_code: u16,
    pub mode: MSMode,
}

impl StationPacket for SetMSModeResponse {
    fn decode(
        buffer: &[u8],
    ) -> color_eyre::eyre::Result<Self, crate::protocol::errors::DecodePacketError> {
        let mut buffer = BytesMut::from(buffer);
        if buffer.len() < 3 {
            return Err(crate::protocol::errors::DecodePacketError::InputTooShort);
        }

        let station_code = buffer.get_u16_le();
        let mode = MSMode::from_byte(buffer.get_u8());

        return Ok(Self { station_code, mode });
    }
}
