#[derive(Debug, Clone, Copy, num_enum::TryFromPrimitive, PartialEq, Eq)]
#[repr(u8)]
/// Master/Slave (Direct/Remote or just M/S) mode
pub enum MsMode {
    /// aka Direct
    Master = 0x4D,
    /// aka Remote
    Slave = 0x53,
}

impl MsMode {
    pub fn to_u8(&self) -> u8 {
        *self as u8
    }

    pub fn from_u8(x: u8) -> Option<Self> {
        match x {
            x if x == MsMode::Master as u8 => Some(MsMode::Master),
            x if x == MsMode::Slave as u8 => Some(MsMode::Slave),
            _ => None,
        }
    }
}
