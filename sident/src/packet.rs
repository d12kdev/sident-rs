use std::fmt::Debug;

use crate::{
    check_vec_len,
    codec::consts::{ETX, STX},
    crc::crc16,
    errors::DeserializePacketError,
    is_extended_packet_id,
};

/// Packet trait for defining packets
pub trait Packet: Debug {
    /// ID of the packet (command byte/ID of the command)
    const PACKET_ID: u8;
}

#[derive(Debug, Clone)]
pub struct RawPacketBody {
    /// Packet ID
    pub id: u8,
    /// Packet data
    pub data: Vec<u8>,
}

/// Raw packet
///
/// * `Nak` - NAK response
/// * `Body` - Body of the raw packet
#[derive(Debug, Clone)]
pub enum RawPacket {
    Nak,
    Body(RawPacketBody),
}

impl RawPacket {
    pub fn deserialize_packet<T: HostboundPacket>(&self) -> Result<T, DeserializePacketError> {
        match self {
            Self::Nak => return Err(DeserializePacketError::ResponseIsNak),
            Self::Body(body) => T::deserialize(body.data.clone()),
        }
    }
}

/// Trait for **host-bound** packets
///
/// Host-bound = **STATION -> HOST**
pub trait HostboundPacket: Packet + Sized {
    /// If the data length is fixed, then this is the length
    const EXPECTED_DATA_LEN: u8;
    /// Is the data length fixed?
    const EXPECTING_DATA_LEN: bool;
    /// Performs checks before deserialization
    ///
    /// * `data` - Packet data to be checked
    fn deserialize_checks(data: &Vec<u8>) -> Result<(), DeserializePacketError> {
        if Self::EXPECTING_DATA_LEN {
            check_vec_len!(
                data,
                Self::EXPECTED_DATA_LEN,
                DeserializePacketError::WrongDataLen
            );
        }
        return Ok(());
    }
    /// Deserializes the packet
    ///
    /// * `data` - Data of the packet
    fn deserialize(data: Vec<u8>) -> Result<Self, DeserializePacketError>;
}

/// Trait for **station-bound** packets
///
/// Station-bound = **HOST -> STATION**
pub trait StationboundPacket: Packet {
    /// Returns the packet data
    fn payload(&self) -> Vec<u8>;
    /// Serializes the packet to **ready-to-send format**
    fn serialize(&self) -> Vec<u8> {
        let mut buffer: Vec<u8> = Vec::new();

        let payload = self.payload();

        buffer.push(STX);
        buffer.push(Self::PACKET_ID);
        if !is_extended_packet_id(Self::PACKET_ID) {
            // BASE
            // TODO: Use DEL
            buffer.extend_from_slice(&payload);
        } else {
            // EXTENDED
            buffer.push(payload.len() as u8);
            buffer.extend_from_slice(&payload);
            let mut crc_buffer: Vec<u8> = Vec::new();
            crc_buffer.push(Self::PACKET_ID);
            crc_buffer.push(payload.len() as u8);
            crc_buffer.extend_from_slice(&payload);
            buffer.extend_from_slice(&crc16(&crc_buffer).to_be_bytes());
        }
        buffer.push(ETX);
        return buffer;
    }
}
