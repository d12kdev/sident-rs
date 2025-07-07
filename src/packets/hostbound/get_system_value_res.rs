use crate::{
    errors::DeserializePacketError,
    packet::{HostboundPacket, Packet},
};

#[derive(Debug)]
pub struct GetSystemValueResponse {
    pub station_code: u16,
    pub address: u8,
    pub data: Vec<u8>,
}

impl Packet for GetSystemValueResponse {
    const PACKET_ID: u8 = 0x83;
}

impl HostboundPacket for GetSystemValueResponse {
    const EXPECTED_DATA_LEN: u8 = 0;
    const EXPECTING_DATA_LEN: bool = false;

    fn deserialize(data: Vec<u8>) -> Result<Self, crate::errors::DeserializePacketError> {
        Self::deserialize_checks(&data)?;

        if data.len() < 3 {
            return Err(DeserializePacketError::WrongDataLen);
        }

        return Ok(Self {
            station_code: u16::from_be_bytes([data[0], data[1]]),
            address: data[2],
            data: data[3..].to_vec(),
        });
    }
}
