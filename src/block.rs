use crate::define_data_address_and_length;


#[derive(Debug)]
pub enum BlockType {
    Card6Block0,
    CardNewerBlock0
}

// Stores the address and offset for certain type of info
// Source: SI_cards_data_structure_developer.ods and SPORTident.Communcation._cardParseSiacBlock0
define_data_address_and_length! {
    Block0CardNewerAddress {
        Uid => (0x00, 0x04),
        RecordStart => (0x04, 0x04),
        Pointers => (0x14, 0x04),
        Sii => (0x19, 0x03), // first byte is not useful for now

    }
}