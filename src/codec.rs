use std::{
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
    pub const STX: u8 = 0x02;
    pub const ETX: u8 = 0x03;
    pub const ACK: u8 = 0x06;
    pub const NAK: u8 = 0x15;
    pub const DLE: u8 = 0x10;
}

#[derive(Debug, Default)]
pub struct SICodec;

impl SICodec {
    pub fn serialize_packet<P: StationboundPacket>(&self, packet: &P) -> Vec<u8> {
        let mut buffer: Vec<u8> = Vec::new();
        buffer.extend_from_slice(&packet.serialize());
        buffer
    }

    pub async fn deserialize_raw_packet(
        &self,
        data: Vec<u8>,
    ) -> Result<RawPacket, DeserializeRawPacketError> {
        let cursor = Cursor::new(data);
        self.deserialize_raw_packet_reader(cursor, None).await
    }

    pub async fn deserialize_raw_packet_reader<R>(
        &self,
        mut reader: R,
        timeout: Option<Duration>,
    ) -> Result<RawPacket, DeserializeRawPacketError>
    where
        R: AsyncRead + Unpin,
    {
        let timeout = timeout.unwrap_or(Duration::from_secs(10));
        let mut final_buffer: Vec<u8> = Vec::new();
        let mut buf = [0u8; 1];
        let start_time = Instant::now();

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
            if start_time.elapsed() >= timeout {
                return Err(DeserializeRawPacketError::TimedOut);
            }

            match reader.read_exact(&mut buf).await {
                Ok(1) => {
                    let byte = buf[0];
                    final_buffer.push(byte);

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
                                return Err(DeserializeRawPacketError::ParseError);
                            }
                        }
                        ParseState::WaitingForEtx => {
                            if extended_protocol {
                                if byte == ETX {
                                    break;
                                } else {
                                    return Err(DeserializeRawPacketError::ParseError);
                                }
                            } else {
                                todo!()
                            }
                        }
                    }
                },
                Ok(_) => return Err(DeserializeRawPacketError::ParseError),
                Err(e) => return Err(DeserializeRawPacketError::IoError(e)),
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
