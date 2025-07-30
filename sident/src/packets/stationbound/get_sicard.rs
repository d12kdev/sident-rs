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

#[derive(Debug)]
pub struct GetSICard6 {
    pub block_number: u8,
}

impl Packet for GetSICard6 {
    const PACKET_ID: u8 = 0xE1;
}

impl StationboundPacket for GetSICard6 {
    fn payload(&self) -> Vec<u8> {
        vec![self.block_number]
    }
}

#[derive(Debug)]
pub struct GetSICard5;

impl Packet for GetSICard5 {
    const PACKET_ID: u8 = 0xB1;
}

impl StationboundPacket for GetSICard5 {
    fn payload(&self) -> Vec<u8> {
        vec![]
    }
}
