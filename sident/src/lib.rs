pub mod card;
pub mod carddef;
pub mod codec;
pub mod connection;
pub mod crc;
pub mod errors;
pub mod firmware;
pub mod macros;
pub mod packet;
pub mod packets;
pub mod product;
pub mod punch;
pub mod time;

mod baudrate;
pub use baudrate::*;
mod ms_mode;
pub use ms_mode::*;
mod protocol_config;
pub use protocol_config::*;
pub mod addr_len;
mod sys_config;

pub use sys_config::*;

use crate::{card::CardType, codec::SICodecTimeout};

/// Returns if the input packet ID is a packet in extended protocol
///
/// # Example
/// ```
/// use sident::is_extended_packet_id;
///
/// assert!(is_extended_packet_id(0xFE));
/// ```
pub fn is_extended_packet_id(id: u8) -> bool {
    if id < 0x80 || id == 0xC4 {
        return false;
    }
    true
}

/// Array of cards supported by this library
pub const SUPPORTED_CARDS: [CardType; 7] = [
    CardType::ActiveCard,
    CardType::Card11,
    CardType::Card10,
    CardType::Card9,
    CardType::Card8,
    CardType::ComCardPro,
    CardType::ComCardUp,
];

/// Returns default SICodec timeout
pub fn td() -> SICodecTimeout {
    return *connection::TIMEOUT_DEFAULT;
}
