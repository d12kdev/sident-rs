use crate::{
    addr_len::AddrLen,
    packet::{Packet, StationboundPacket},
};

#[derive(Debug)]
pub struct GetSystemValue {
    pub addr_len: AddrLen,
}

impl Packet for GetSystemValue {
    const PACKET_ID: u8 = 0x83;
}

impl StationboundPacket for GetSystemValue {
    fn payload(&self) -> Vec<u8> {
        vec![self.addr_len.address_byte(), self.addr_len.length_byte()]
    }
}
