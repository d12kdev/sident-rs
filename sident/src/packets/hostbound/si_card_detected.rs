use crate::packet::{HostboundPacket, Packet};

#[derive(Debug)]
pub struct SICardNewerDetected {
    pub station_code: u16,
    pub siid: u32,
}

impl Packet for SICardNewerDetected {
    const PACKET_ID: u8 = 0xE8;
}

impl HostboundPacket for SICardNewerDetected {
    const EXPECTED_DATA_LEN: u8 = 6;
    const EXPECTING_DATA_LEN: bool = true;

    fn deserialize(data: Vec<u8>) -> Result<Self, crate::errors::DeserializePacketError> {
        Self::deserialize_checks(&data)?;

        return Ok(Self {
            station_code: u16::from_be_bytes([data[0], data[1]]),
            // SI3 is ignored, because it wont give the correct card number. sportident docs are just weird...
            siid: u32::from_be_bytes([0, data[3], data[4], data[5]]),
        });
    }
}

#[derive(Debug)]
pub struct SICard6Detected {
    pub station_code: u16,
    pub siid: u32,
}

impl Packet for SICard6Detected {
    const PACKET_ID: u8 = 0xE6;
}

impl HostboundPacket for SICard6Detected {
    const EXPECTED_DATA_LEN: u8 = 6;
    const EXPECTING_DATA_LEN: bool = true;
    fn deserialize(data: Vec<u8>) -> Result<Self, crate::errors::DeserializePacketError> {
        Self::deserialize_checks(&data)?;

        return Ok(Self {
            station_code: u16::from_be_bytes([data[0], data[1]]),
            // SI3 is ignored, because it wont give the correct card number. sportident docs are just weird...
            siid: u32::from_be_bytes([0, data[3], data[4], data[5]]),
        });
    }
}


#[derive(Debug)]
pub struct SICard5Detected {
    pub station_code: u16,
    pub siid: u32
}

impl Packet for SICard5Detected {
    const PACKET_ID: u8 = 0xE5;
}

impl HostboundPacket for SICard5Detected {
    const EXPECTED_DATA_LEN: u8 = 6;
    const EXPECTING_DATA_LEN: bool = true;
    fn deserialize(data: Vec<u8>) -> Result<Self, crate::errors::DeserializePacketError> {
        Self::deserialize_checks(&data)?;

        let station_code = u16::from_be_bytes([data[0], data[1]]);
        let siid = u32::from_be_bytes([0, data[3], data[4], data[5]]);

        return Ok(
            Self {
                station_code,
                siid
            }
        )
    }
}