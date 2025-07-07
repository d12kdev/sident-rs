use crate::packet::{Packet, StationboundPacket};


#[derive(Debug)]
pub struct GetSICardNewer {
    pub block_number: u8, // TODO: maybe enum?
}

impl Packet for GetSICardNewer {
    const PACKET_ID: u8 = 0xEF;
}

impl StationboundPacket for GetSICardNewer {
    fn payload(&self) -> Vec<u8> {
        vec![self.block_number]
    }
}