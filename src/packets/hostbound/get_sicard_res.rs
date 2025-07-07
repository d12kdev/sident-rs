use crate::packet::{HostboundPacket, Packet};


#[derive(Debug)]
pub struct GetSICardNewerResponse {
    pub station_code: u16,
    pub block_number: u8, // TODO: maybe enum? 2
    pub data: [u8; 128]
}

impl Packet for GetSICardNewerResponse {
    const PACKET_ID: u8 = 0xEF;
}

impl HostboundPacket for GetSICardNewerResponse {
    const EXPECTED_DATA_LEN: u8 = 131;
    const EXPECTING_DATA_LEN: bool = true;
    fn deserialize(data: Vec<u8>) -> Result<Self, crate::errors::DeserializePacketError> {
        Self::deserialize_checks(&data)?;
        
        return Ok(
            Self {
                station_code: u16::from_be_bytes([data[0], data[1]]),
                block_number: data[2],
                data: data[3..].try_into().unwrap() // should not fail because of the checks
            }
        )
    }
}