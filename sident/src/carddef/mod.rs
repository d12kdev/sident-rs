use crate::{
    card::CardPersonalData,
    errors::{DeserializeCardPersonalDataError, FeedBlockError},
    punch::Punch,
};
use std::fmt::Debug;

pub mod si10;
pub mod si8;
pub mod siac;

pub mod si11 {
    pub type Card11Def = super::si10::Card10Def;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BlockNeededIntention {
    CardPersonalData,
    Punches,
    CardExclusives,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BlockNeededResult {
    Need(u8),
    NoNeed,
}

pub trait CardDefinition: Debug {
    const HAS_CARD_EXCLUSIVES: bool = false;
    type CardExclusivesType;
    fn get_exclusives(&self) -> Option<Self::CardExclusivesType> {
        return None;
    }
    fn has_exclusives(&self) -> bool {
        return Self::HAS_CARD_EXCLUSIVES;
    }

    fn new_empty() -> Self;
    fn get_siid(&self) -> Option<u32>;
    fn get_punch_count(&self) -> Option<u8>;
    /// Note that SI5 does not support CardPersonalData
    fn get_personal_data(
        &self,
    ) -> Option<Result<CardPersonalData, DeserializeCardPersonalDataError>>;
    fn get_clear_check(&self) -> Option<Punch>;
    fn get_start(&self) -> Option<Option<Punch>>;
    fn get_finish(&self) -> Option<Option<Punch>>;
    fn feed_block(&mut self, block_id: u8, block_buffer: &[u8; 128]) -> Result<(), FeedBlockError>;
    /// Why read every block when you can ask the CardDef what block is needed?
    fn block_needed(&self, intention: &BlockNeededIntention) -> BlockNeededResult;
    fn has_block(&self, block_id: u8) -> bool;
    fn get_punches(&self) -> Option<Vec<Punch>>;
    // TODO: more methods
}
