use super::errors::DecodePacketError;
use bytes::BytesMut;
use color_eyre::eyre::Result;

pub trait Packet {
    const PACKET_ID: u8;
}

pub trait HostPacket: Packet {
    fn write(&self, buffer: &mut BytesMut);
}

pub trait StationPacket: Packet + Sized {
    fn decode(buffer: &[u8]) -> Result<Self, DecodePacketError>;
}

macro_rules! pckt {
    ($structname: ident, $packetid: expr) => {
        impl $crate::protocol::Packet for $structname {
            const PACKET_ID: u8 = $packetid;
        }
    };
}

pub(crate) use pckt;
