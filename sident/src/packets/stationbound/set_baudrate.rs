use crate::{
    Baudrate,
    packet::{Packet, StationboundPacket},
};

/// Set baudrate
#[derive(Debug)]
pub struct SetBaudrate {
    pub baudrate: Baudrate,
}

impl Packet for SetBaudrate {
    const PACKET_ID: u8 = 0xFE;
}

impl StationboundPacket for SetBaudrate {
    fn payload(&self) -> Vec<u8> {
        vec![self.baudrate.to_u8()]
    }
}

/// Set baudrate for base protocol
#[derive(Debug)]
pub struct BaseSetBaudrate {
    pub baudrate: Baudrate,
}

impl Packet for BaseSetBaudrate {
    const PACKET_ID: u8 = 0x7E;
}

impl StationboundPacket for BaseSetBaudrate {
    fn payload(&self) -> Vec<u8> {
        vec![self.baudrate.to_u8()]
    }
}
