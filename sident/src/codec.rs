use std::{
    collections::HashMap,
    io::Cursor,
    time::{Duration, Instant},
};

use tokio::io::{AsyncRead, AsyncReadExt};

use crate::{
    codec::consts::{ETX, NAK, STX},
    crc::crc16,
    errors::DeserializeRawPacketError,
    is_extended_packet_id,
    packet::{RawPacket, RawPacketBody, StationboundPacket},
};

pub mod consts {
    /// Start of text - first byte transmitted
    pub const STX: u8 = 0x02;
    /// End of text - last byte transmitted
    pub const ETX: u8 = 0x03;
    /// Positive handshake result
    pub const ACK: u8 = 0x06;
    /// Negative handshake result
    pub const NAK: u8 = 0x15;
    /// Delimiter inserted before data characters 00-1F (base protocol)
    pub const DLE: u8 = 0x10;
}

#[derive(Debug, Clone, Copy)]
pub enum SICodecTimeout {
    Infinite,
    Finite(Duration),
}

/// Codec utils for SPORTident
#[derive(Debug, Default)]
pub struct SICodec;

impl SICodec {
    pub fn replace_printer_charset_bytes(data: &[u8]) -> Vec<u8> {
        let map = HashMap::from([
            (0x80, 0xC7),
            (0x81, 0xFC),
            (0x82, 0xE9),
            (0x83, 0xE2),
            (0x84, 0xE4),
            (0x85, 0xE0),
            (0x86, 0xE5),
            (0x87, 0xE7),
            (0x88, 0xEA),
            (0x89, 0xEB),
            (0x8A, 0xE8),
            (0x8B, 0xEF),
            (0x8C, 0xEE),
            (0x8D, 0xEC),
            (0x8E, 0xC4),
            (0x8F, 0xC5),
            (0x90, 0xC9),
            (0x91, 0xE6),
            (0x92, 0xC6),
            (0x93, 0xF4),
            (0x94, 0xF6),
            (0x95, 0xF2),
            (0x96, 0xFB),
            (0x97, 0xF9),
            (0x98, 0xFF),
            (0x99, 0xD6),
            (0x9A, 0xDC),
            (0xA0, 0xE1),
            (0xA1, 0xED),
            (0xA2, 0xF3),
            (0xA3, 0xFA),
            (0xA4, 0xF1),
            (0xA5, 0xD1),
            (0xE1, 0xDF),
        ]);

        data.iter().map(|b| *map.get(b).unwrap_or(b)).collect()
    }

    pub fn encode_iso_8859_1(s: &str) -> Option<Vec<u8>> {
        s.chars()
            .map(|c| {
                let code = c as u32;
                if code <= 0xFF { Some(code as u8) } else { None }
            })
            .collect()
    }

    pub fn decode_iso_8859_1(data: &[u8]) -> Result<String, std::string::FromUtf8Error> {
        let mut out = vec![0u8; data.len() * 2];
        let written = encoding_rs::mem::convert_latin1_to_utf8(&data, &mut out);
        out.truncate(written);
        return String::from_utf8(out);
    }

    /// Alias for `StationboundPacket::serialize()`
    ///
    /// * `packet` - Packet to be serialized
    pub fn serialize_packet<P: StationboundPacket>(packet: &P) -> Vec<u8> {
        packet.serialize()
    }

    /// Deserializes raw packet from data instead of reader
    ///
    /// * `data` - Data
    pub async fn deserialize_raw_packet(
        data: &[u8],
    ) -> Result<RawPacket, DeserializeRawPacketError> {
        let cursor = Cursor::new(data);
        Self::deserialize_raw_packet_reader(
            cursor,
            SICodecTimeout::Finite(Duration::from_millis(1)),
            SICodecTimeout::Finite(Duration::from_millis(1)),
        )
        .await
    }

    /// Deserializes raw packet from async reader
    ///
    /// * `reader` - Reader (**async**)
    /// * `stx_timeout` - Timeout for the `STX` byte
    /// * `timeout` - Timeout for other bytes
    pub async fn deserialize_raw_packet_reader<R>(
        mut reader: R,
        stx_timeout: SICodecTimeout,
        timeout: SICodecTimeout,
    ) -> Result<RawPacket, DeserializeRawPacketError>
    where
        R: AsyncRead + Unpin,
    {
        async fn read_w_timeout<R: AsyncRead + Unpin>(
            reader: &mut R,
            buf: &mut [u8],
            timeout: &SICodecTimeout,
        ) -> Result<usize, DeserializeRawPacketError> {
            match timeout {
                SICodecTimeout::Finite(dur) => {
                    match tokio::time::timeout(*dur, reader.read_exact(buf)).await {
                        Ok(Ok(n)) => Ok(n),
                        Ok(Err(e)) => Err(DeserializeRawPacketError::IoError(e)),
                        Err(_) => Err(DeserializeRawPacketError::TimedOut),
                    }
                }
                SICodecTimeout::Infinite => reader
                    .read_exact(buf)
                    .await
                    .map_err(|e| DeserializeRawPacketError::IoError(e)),
            }
        }

        let mut final_buffer: Vec<u8> = Vec::new();
        let mut buf = [0u8; 1];
        let mut byte_time = Instant::now();

        #[derive(PartialEq, Eq)]
        enum ParseState {
            WaitingForStart,
            ReadingHeader,
            ReadingData {
                bytes_to_read: usize,
                bytes_read: usize,
            },
            ReadingCrc,
            WaitingForEtx,
        }

        let mut state = ParseState::WaitingForStart;
        let mut expected_data_len: usize = 0;

        let mut result_packet_body = RawPacketBody {
            id: 0,
            data: vec![],
        };

        let mut extended_protocol = true;
        let mut crc_bytes = [0u8; 2];
        let mut crc = 0u16;

        loop {
            if let SICodecTimeout::Finite(timeout) = timeout {
                if state != ParseState::WaitingForStart && byte_time.elapsed() >= timeout {
                    log::error!("timeout");
                    return Err(DeserializeRawPacketError::TimedOut);
                }
            }

            if let SICodecTimeout::Finite(timeout) = stx_timeout {
                if state == ParseState::WaitingForStart && byte_time.elapsed() >= timeout {
                    log::error!("stxtimeout");
                    return Err(DeserializeRawPacketError::TimedOut);
                }
            }

            let read_timeout = if state == ParseState::WaitingForStart {
                &stx_timeout
            } else {
                &timeout
            };

            match read_w_timeout(&mut reader, &mut buf, read_timeout).await {
                Ok(1) => {
                    let byte = buf[0];
                    final_buffer.push(byte);
                    byte_time = Instant::now();

                    match &mut state {
                        ParseState::WaitingForStart => {
                            if byte == STX {
                                final_buffer.clear();
                                final_buffer.push(byte);
                                state = ParseState::ReadingHeader;
                            } else if byte == NAK {
                                return Ok(RawPacket::Nak);
                            }
                        }
                        ParseState::ReadingHeader => {
                            if final_buffer.len() == 2 {
                                let packet_id = final_buffer[1];
                                result_packet_body.id = packet_id;

                                if !is_extended_packet_id(packet_id) {
                                    extended_protocol = false;
                                    state = ParseState::WaitingForEtx;
                                }
                            }

                            if final_buffer.len() == 3 {
                                let len_byte = final_buffer[2];
                                expected_data_len = len_byte as usize;
                                state = ParseState::ReadingData {
                                    bytes_to_read: expected_data_len,
                                    bytes_read: 0,
                                }
                            }
                        }
                        ParseState::ReadingData {
                            bytes_to_read,
                            bytes_read,
                        } => {
                            assert!(extended_protocol);
                            if *bytes_to_read > 0 {
                                *bytes_read += 1;
                                result_packet_body.data.push(byte);
                                if *bytes_read == *bytes_to_read {
                                    state = ParseState::ReadingCrc;
                                }
                            } else {
                                state = ParseState::ReadingCrc;
                            }
                        }
                        ParseState::ReadingCrc => {
                            assert!(extended_protocol);
                            let len = 3 + expected_data_len;
                            if final_buffer.len() == len + 1 {
                                crc_bytes[0] = byte;
                            } else if final_buffer.len() == len + 2 {
                                crc_bytes[1] = byte;
                                crc = u16::from_be_bytes(crc_bytes);
                                state = ParseState::WaitingForEtx;
                            } else {
                                log::error!("Failed to deserialize CRC");
                                return Err(DeserializeRawPacketError::ParseError);
                            }
                        }
                        ParseState::WaitingForEtx => {
                            if extended_protocol {
                                if byte == ETX {
                                    break;
                                } else {
                                    log::error!("Failed to wait for ETX");
                                    return Err(DeserializeRawPacketError::ParseError);
                                }
                            } else {
                                todo!()
                            }
                        }
                    }
                }
                Ok(_) => return Err(DeserializeRawPacketError::ParseError),
                Err(e) => return Err(e),
            }
        }

        if extended_protocol {
            let mut crc_buffer = Vec::new();
            crc_buffer.push(result_packet_body.id);
            crc_buffer.push(expected_data_len as u8);
            crc_buffer.extend_from_slice(&result_packet_body.data);

            if crc16(&crc_buffer) != crc {
                return Err(DeserializeRawPacketError::CrcError);
            }
        }

        Ok(RawPacket::Body(result_packet_body))
    }
}
