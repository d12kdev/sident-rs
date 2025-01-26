pub mod host;
pub mod station;

mod packet;
pub use packet::*;

pub mod decoder;
pub mod encoder;
pub mod errors;

mod response;
pub use response::*;

mod crc;
pub use crc::*;

pub mod consts;

#[derive(Debug, PartialEq, Eq)]
pub enum Baudrate {
    High,
    Low,
}

impl Baudrate {
    pub fn to_byte(&self) -> u8 {
        match self {
            Self::High => 1,
            Self::Low => 0,
        }
    }

    pub fn from_byte(byte: u8) -> Option<Self> {
        match byte {
            1 => Some(Self::High),
            0 => Some(Self::Low),
            _ => None,
        }
    }

    pub fn actual_speed(&self) -> u32 {
        match self {
            Self::High => consts::HIGH_BAUDRATE,
            Self::Low => consts::LOW_BAUDRATE,
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum ProtocolMode {
    Base,
    Extended,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum MSMode {
    Master,
    Slave,
}

impl MSMode {
    pub fn to_byte(&self) -> u8 {
        match self {
            MSMode::Master => 0x4D,
            MSMode::Slave => 0x53,
        }
    }

    pub fn from_byte(byte: u8) -> Self {
        match byte {
            0x4D => MSMode::Master,
            0x53 => MSMode::Slave,
            _ => MSMode::Master,
        }
    }
}
