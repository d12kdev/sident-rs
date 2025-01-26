use crate::protocol::{pckt, HostPacket};


pckt!(EraseBackupData, 0xF5);
#[derive(Debug)]
pub struct EraseBackupData;

impl HostPacket for EraseBackupData {
    fn write(&self, buffer: &mut bytes::BytesMut) {
        // NO DATA
    }
}