use bytes::BufMut;

use crate::protocol::{pckt, Baudrate, HostPacket};

pckt!(SetBaudrate, 0xFE);
#[derive(Debug)]
pub struct SetBaudrate {
    baudrate: Baudrate,
}

impl SetBaudrate {
    pub fn new(baudrate: Baudrate) -> Self {
        Self { baudrate }
    }
}

impl HostPacket for SetBaudrate {
    fn write(&self, buffer: &mut bytes::BytesMut) {
        buffer.put_u8(self.baudrate.to_byte());
    }
}
