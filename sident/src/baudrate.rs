#[derive(Debug, Clone, Copy, PartialEq, Eq, num_enum::TryFromPrimitive)]
#[repr(u8)]
pub enum Baudrate {
    /// **38400** baud
    High = 1,
    /// **4800** baud
    Low = 0,
}

impl Baudrate {
    pub fn to_u8(&self) -> u8 {
        *self as u8
    }

    pub fn actual_baudrate(&self) -> u32 {
        match self {
            Self::High => 38400,
            Self::Low => 4800,
        }
    }
}
