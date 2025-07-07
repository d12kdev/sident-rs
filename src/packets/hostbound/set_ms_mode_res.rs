use crate::{
    MsMode,
    packet::{HostboundPacket, Packet},
};

#[derive(Debug)]
pub struct SetMsModeResponse {
    pub station_code: u16,
    pub mode: MsMode,
}

impl Packet for SetMsModeResponse {
    const PACKET_ID: u8 = 0xF0;
}

impl HostboundPacket for SetMsModeResponse {
    const EXPECTED_DATA_LEN: u8 = 3;
    const EXPECTING_DATA_LEN: bool = true;
    fn deserialize(data: Vec<u8>) -> Result<Self, crate::errors::DeserializePacketError> {
        Self::deserialize_checks(&data)?;

        return Ok(Self {
            station_code: u16::from_be_bytes([data[0], data[1]]),
            mode: MsMode::from_u8(data[2]).ok_or(crate::errors::DeserializePacketError::Other(
                format!("MsMode is invalid"),
            ))?,
        });
    }
}
