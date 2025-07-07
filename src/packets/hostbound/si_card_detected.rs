use crate::packet::{HostboundPacket, Packet};


#[derive(Debug)]
pub struct SICardNewerDetected {
    pub station_code: u16,
    pub card_nunmber: u32
}

impl Packet for SICardNewerDetected {
    const PACKET_ID: u8 = 0xE8;
}

impl HostboundPacket for SICardNewerDetected {
    const EXPECTED_DATA_LEN: u8 = 6;
    const EXPECTING_DATA_LEN: bool = true;

    fn deserialize(data: Vec<u8>) -> Result<Self, crate::errors::DeserializePacketError> {
        Self::deserialize_checks(&data)?;

        return Ok(
            Self {
                station_code: u16::from_be_bytes([data[0], data[1]]),
                // SI3 is ignored, because it wont give the correct card number. sportident docs are just weird...
                card_nunmber: u32::from_be_bytes([0, data[3], data[4], data[5]])
            }
        )    
    }
}