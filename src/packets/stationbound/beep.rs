use crate::packet::{Packet, StationboundPacket};

#[derive(Debug)]
pub struct BeepIfStationReady {
    pub beep_count: u8,
}

impl Packet for BeepIfStationReady {
    const PACKET_ID: u8 = 0xF9;
}

impl StationboundPacket for BeepIfStationReady {
    fn payload(&self) -> Vec<u8> {
        vec![self.beep_count]
    }
}

#[derive(Debug)]
pub struct BeepIfCardReady;

impl Packet for BeepIfCardReady {
    const PACKET_ID: u8 = 0x06;
}

impl StationboundPacket for BeepIfCardReady {
    fn payload(&self) -> Vec<u8> {
        vec![]
    }
}