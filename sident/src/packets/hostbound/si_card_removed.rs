use crate::packet::{HostboundPacket, Packet};

#[derive(Debug)]
pub struct SICardRemoved {
    pub station_code: u16,
    pub siid: u32,
}

impl Packet for SICardRemoved {
    const PACKET_ID: u8 = 0xE7;
}

impl HostboundPacket for SICardRemoved {
    const EXPECTED_DATA_LEN: u8 = 6;
    const EXPECTING_DATA_LEN: bool = true;
    fn deserialize(data: Vec<u8>) -> Result<Self, crate::errors::DeserializePacketError> {
        Self::deserialize_checks(&data)?;

        let station_code = u16::from_be_bytes([data[0], data[1]]);
        let siid = u32::from_be_bytes([0, data[3], data[4], data[5]]);

        return Ok(Self { station_code, siid });
    }
}


#[derive(Debug)]
pub struct SICard6Removed {
    pub station_code: u16,
    pub siid: u32
}

impl Packet for SICard6Removed {
    const PACKET_ID: u8 = 0xE7;
}

impl HostboundPacket for SICard6Removed {
    const EXPECTED_DATA_LEN: u8 = 6;
    const EXPECTING_DATA_LEN: bool = true;

    fn deserialize(data: Vec<u8>) -> Result<Self, crate::errors::DeserializePacketError> {
        Self::deserialize_checks(&data)?;

        let station_code = u16::from_be_bytes([data[0], data[1]]);
        let siid = u32::from_be_bytes([0, data[3], data[4], data[5]]);

        Ok(
            Self {
                station_code,
                siid
            }
        )
    }
}


#[derive(Debug)]
pub struct SICard5Removed {
    pub station_code: u16,
    pub sii: u32
}

impl Packet for SICard5Removed {
    const PACKET_ID: u8 = 0xE7;
}

impl HostboundPacket for SICard5Removed {
    const EXPECTED_DATA_LEN: u8 = 6;
    const EXPECTING_DATA_LEN: bool = true;

    fn deserialize(data: Vec<u8>) -> Result<Self, crate::errors::DeserializePacketError> {
        Self::deserialize_checks(&data)?;

        let station_code = u16::from_be_bytes([data[0], data[1]]);
        let sii = u32::from_be_bytes([0, data[3], data[4], data[5]]);

        Ok(
            Self {
                station_code,
                sii
            }
        )
    }
}