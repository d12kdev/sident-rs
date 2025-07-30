use crate::{
    MsMode,
    packet::{Packet, StationboundPacket},
};

#[derive(Debug)]
pub struct SetMsMode {
    pub mode: MsMode,
}

impl Packet for SetMsMode {
    const PACKET_ID: u8 = 0xF0;
}

impl StationboundPacket for SetMsMode {
    fn payload(&self) -> Vec<u8> {
        vec![self.mode.to_u8()]
    }
}

#[derive(Debug)]
pub struct BaseSetMsMode {
    pub mode: MsMode,
}

impl Packet for BaseSetMsMode {
    const PACKET_ID: u8 = 0x70;
}

impl StationboundPacket for BaseSetMsMode {
    fn payload(&self) -> Vec<u8> {
        vec![self.mode.to_u8()]
    }
}
