use bytes::BytesMut;

pub trait Packet {
    const PACKET_ID: u8;
}

pub trait HostPacket: Packet {
    fn write(&self, buffer: &mut BytesMut);
}

macro_rules! pckt {
    ($structname: ident, $packetid: expr) => {
        impl $crate::protocol::Packet for $structname {
            const PACKET_ID: u8 = $packetid;
        }
    };
}

pub(crate) use pckt;