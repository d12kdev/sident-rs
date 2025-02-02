use bytes::{Buf, BufMut, BytesMut};
use color_eyre::eyre::Result;

use crate::protocol::calculate_crc;

use super::{
    consts::{ETX, NAK, STX},
    encoder::Encoder,
    errors::DecoderError,
    Response,
};

#[derive(Debug)]
pub struct Decoder;

impl Decoder {
    pub fn new() -> Self {
        Self
    }

    pub fn decode(&self, input: &[u8]) -> Result<Response, DecoderError> {
        let mut input = BytesMut::from(input);
        // base:
        // STX
        // command_code
        // data
        // ETX

        // extended:
        // STX
        // command_code
        // data_len
        // data
        // crc (2 bytes)
        // ETX

        if input.len() < 4 {
            if input.len() == 1 && input.get_u8() == NAK {
                return Ok(Response::new_nak());
            }
            return Err(DecoderError::InputTooShort);
        }

        match input.get_u8() {
            STX => (),
            byte => return Err(DecoderError::WrongStartByte(byte)),
        }

        let command_code = input.get_u8();

        if Encoder::is_extended_instruction(&command_code) {
            if input.len() < 5 {
                return Err(DecoderError::InputTooShort);
            }

            let data_len = input.get_u8() as usize;

            // +3 because CRC1 CRC0 ETX
            if input.len() < (data_len + 3) {
                return Err(DecoderError::InputTooShort);
            }

            let data = input.split_to(data_len);

            if input.len() > 3 {
                return Err(DecoderError::InputTooLong);
            }

            let crc = input.get_u16_le();

            match input.get_u8() {
                ETX => (),
                byte => return Err(DecoderError::WrongEndByte(byte)),
            }

            let mut crc_in = BytesMut::new();
            crc_in.put_u8(data_len as u8);
            crc_in.extend_from_slice(&data);

            if calculate_crc(&crc_in) == crc {
                return Ok(Response::new(
                    super::ProtocolMode::Extended,
                    command_code,
                    data,
                ));
            } else {
                return Err(DecoderError::CrcDoesNotMatch);
            }
        } else {
            return Err(DecoderError::LegacyNotSupported);
        }
    }
}

#[cfg(test)]
mod tests {
    use bytes::BytesMut;

    use crate::protocol::{
        consts::{ETX, STX},
        decoder::Decoder,
        station, Baudrate, Response,
    };

    #[test]
    fn test_decode() {
        let bytes_vec: Vec<u8> = vec![STX, 0xF0, 0x03, 0x01, 0x02, 0x53, 0xFE, 0x31, ETX];
        let decoded = Decoder::new().decode(&bytes_vec).unwrap();
        let _assert_bytes: &[u8] = &vec![0x01, 0x02, 0x53];
        assert_eq!(
            decoded,
            Response::new(
                crate::protocol::ProtocolMode::Extended,
                0xF0,
                BytesMut::from(_assert_bytes)
            )
        );
    }

    #[test]
    fn test_decode_set_baudrate_response() {
        let bytes_vec: Vec<u8> = vec![STX, 0xFE, 0x03, 0x00, 0x00, 0x01, 0x05, 0xBC, ETX];
        let decoded: station::SetBaudrateResponse = Decoder::new()
            .decode(&bytes_vec)
            .unwrap()
            .into_packet()
            .unwrap();
        assert_eq!(decoded.baudrate, Baudrate::High);
        assert_eq!(decoded.station_code, 0);
    }
}
