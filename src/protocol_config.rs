use bitflags::bitflags;

bitflags! {
    #[derive(Debug)]
    pub struct ProtocolConfig: u8 {
        const EXTENDED_PROTOCOL =       0b0000_0001;
        const AUTO_SEND_OUT =           0b0000_0010;
        const HANDSHAKE =               0b0000_0100;
        const PASSWORD_ONLY =           0b0001_0000;
        const READ_OUT_AFTER_PUNCH =    0b1000_0000;
    }
}
