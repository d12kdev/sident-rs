pub mod base;
pub mod extended;

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
pub enum ProtocolMode {
    Base,
    Extended,
}

#[derive(Debug, PartialEq, Eq)]
pub enum MSMode {
    Master,
    Slave
}

impl MSMode {
    pub fn to_byte(&self) -> u8 {
        match self {
            MSMode::Master => 0x4D,
            MSMode::Slave => 0x53
        }
    }
}