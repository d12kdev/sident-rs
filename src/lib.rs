pub mod codec;
pub mod connection;
pub mod crc;
pub mod errors;
pub mod firmware;
pub mod macros;
pub mod packet;
pub mod packets;
pub mod product;
pub mod time;
pub mod card;
pub mod block;
pub mod punch;

mod baudrate;
pub use baudrate::*;
mod ms_mode;
pub use ms_mode::*;
mod protocol_config;
pub use protocol_config::*;
mod data_addr_len;
pub use data_addr_len::*;
mod sys_config;
pub use sys_config::*;

pub fn is_extended_packet_id(id: u8) -> bool {
    if id < 0x80 || id == 0xC4 {
        return false;
    }
    true
}
