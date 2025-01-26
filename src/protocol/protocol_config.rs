#[derive(Debug)]
pub struct ProtocolConfig {
    pub extended_protocol: bool,
    pub auto_send_out: bool,
    pub handshake: bool,
    pub access_with_password_only: bool,
    pub read_out_after_punch: bool,
}

impl ProtocolConfig {
    const EXTENDED_PROTOCOL: u8 = 0b_00000001;
    const AUTO_SEND_OUT: u8 = 0b_00000010;
    const HANDSHAKE: u8 = 0b_00000100;
    const ACCESS_PASSWORD: u8 = 0b_00010000;
    const READOUT_AFTER_PUNCH: u8 = 0b_10000000;

    pub fn from_byte(byte: u8) -> Self {
        fn val(byte: u8, mask: u8) -> bool {
            return byte & mask != 0;
        }

        return Self {
            extended_protocol: val(byte, Self::EXTENDED_PROTOCOL),
            auto_send_out: val(byte, Self::AUTO_SEND_OUT),
            handshake: val(byte, Self::HANDSHAKE),
            access_with_password_only: val(byte, Self::ACCESS_PASSWORD),
            read_out_after_punch: val(byte, Self::READOUT_AFTER_PUNCH),
        };
    }

    pub fn to_byte(&self) -> u8 {
        let mut byte = 0;

        if self.extended_protocol {
            byte |= Self::EXTENDED_PROTOCOL;
        }

        if self.auto_send_out {
            byte |= Self::AUTO_SEND_OUT;
        }

        if self.handshake {
            byte |= Self::HANDSHAKE;
        }

        if self.access_with_password_only {
            byte |= Self::ACCESS_PASSWORD;
        }

        if self.read_out_after_punch {
            byte |= Self::READOUT_AFTER_PUNCH;
        }

        return byte;
    }
}
