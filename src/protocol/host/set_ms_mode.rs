use bytes::BufMut;

use crate::protocol::{pckt, HostPacket, MSMode};

pckt!(SetMSMode, 0xF0);
#[derive(Debug)]
pub struct SetMSMode {
    mode: MSMode,
}

impl SetMSMode {
    pub fn new(mode: MSMode) -> Self {
        Self { mode }
    }
}

impl HostPacket for SetMSMode {
    fn write(&self, buffer: &mut bytes::BytesMut) {
        buffer.put_u8(self.mode.to_byte());
    }
}
