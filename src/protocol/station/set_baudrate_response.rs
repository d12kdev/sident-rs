use bytes::{Buf, BytesMut};

use crate::protocol::{pckt, Baudrate, StationPacket};

pckt!(SetBaudrateResponse, 0xFE);
#[derive(Debug)]
pub struct SetBaudrateResponse {
    pub station_code: u16,
    pub baudrate: Baudrate,
}

impl StationPacket for SetBaudrateResponse {
    fn decode(
        buffer: &[u8],
    ) -> color_eyre::eyre::Result<Self, crate::protocol::errors::DecodePacketError> {
        let mut buffer = BytesMut::from(buffer);
        if buffer.len() < 3 {
            return Err(crate::protocol::errors::DecodePacketError::InputTooShort);
        }

        let station_code = buffer.get_u16_le();
        let baudrate_byte = buffer.get_u8();
        let baudrate = match Baudrate::from_byte(baudrate_byte) {
            Some(some) => some,
            None => return Err(crate::protocol::errors::DecodePacketError::WrongData),
        };

        return Ok(Self {
            station_code,
            baudrate,
        });
    }
}
