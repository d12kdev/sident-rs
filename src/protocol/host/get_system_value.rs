use bytes::BufMut;

use crate::protocol::{pckt, HostPacket};

pckt!(GetSystemValue, 0x83);
#[derive(Debug)]
pub struct GetSystemValue {
    /// The address/offset
    address: u8,
    /// The desired length of the returned data
    length: u8, // SPORTident.Communication.dll/SPORTident.Communication/Communication/_cmdGetSysData
}

impl HostPacket for GetSystemValue {
    fn write(&self, buffer: &mut bytes::BytesMut) {
        buffer.put_u8(self.address);
        buffer.put_u8(self.length);
    }
}
