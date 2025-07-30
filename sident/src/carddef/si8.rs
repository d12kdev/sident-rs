// TODO: REWRITE WITH SI DOCS
/*
    SPORTident Card-8 Memory Structure

    Every integer is big endian encoded unless said differently in the desc.

    ---- BLOCK 0x00 ----
    0x00..0x03  Unique device ID (PROBABLY, IDK)
    0x04..0x07  Start of record - 0xEA 0xEA 0xEA 0xEA
    0x08..0x0B  Clear/Check (punch)
    0x0C..0x0F  Start (punch)
    0x10..0x13  Finish (punch)
    0x16        Punch pointer - punch count
    0x18..0x1B  SIID - SI3, SI2, SI1, SI0 - SI2..SI0 makes up the SIID (card number)
    0x20..0x7F  Card personal data - PART1
    ----            ----
    ---- BLOCK 0x01 ----
    0x00..0x07  Card personal data - PART2
    0x08..0x7F  Punches
    ----            ----
*/

use crate::{
    card::CardPersonalData,
    carddef::{BlockNeededIntention, BlockNeededResult, CardDefinition},
    errors::{DeserializeBlockError, DeserializeCardPersonalDataError},
    punch::Punch,
};

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(export, export_to = "SI8Block0.ts"))]
#[derive(Debug, Clone)]
struct Block0 {
    _uid: u32,
    clear_check: Punch,
    start: Option<Punch>,
    finish: Option<Punch>,
    punch_count: u8,
    siid: u32,
    #[cfg_attr(feature = "serde", serde(with = "serde_big_array::BigArray"))]
    #[cfg_attr(feature = "ts-rs", ts(type = "[number; 96]"))]
    card_personal_data1: [u8; 96],
    personal_data_finished: bool,
}

impl Block0 {
    pub fn deserialize(data: [u8; 128]) -> Result<Self, DeserializeBlockError> {
        let uid = u32::from_be_bytes([data[0], data[1], data[2], data[3]]);
        let clear_check = Punch::deserialize(&[data[8], data[9], data[10], data[11]])?;
        let start = Punch::deserialize_control_punch(&[data[12], data[13], data[14], data[15]])?;
        let finish = Punch::deserialize_control_punch(&[data[16], data[17], data[18], data[19]])?;
        let punch_count = data[22];
        let siid = u32::from_be_bytes([0, data[25], data[26], data[27]]);
        let card_personal_data1: [u8; 96] = data[32..].try_into().unwrap(); // cant fail 
        let personal_data_finished =
            [card_personal_data1[94], card_personal_data1[95]] == [0x00, 0x00];

        return Ok(Self {
            _uid: uid,
            clear_check,
            start,
            finish,
            punch_count,
            siid,
            card_personal_data1,
            personal_data_finished,
        });
    }
}

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(export, export_to = "SI8Block1.ts"))]
#[derive(Debug, Clone)]
struct Block1 {
    card_personal_data2: [u8; 8],
    punches: Vec<Punch>,
}

impl Block1 {
    pub fn deserialize(data: [u8; 128]) -> Result<Self, DeserializeBlockError> {
        let card_personal_data2: [u8; 8] = data[..8].try_into().unwrap();
        let punches_bytes: [u8; 120] = data[8..].try_into().unwrap();

        let mut punches: Vec<Punch> = Vec::new();
        for punch_chunk in punches_bytes.chunks(4) {
            let punch_chunk: &[u8; 4] = punch_chunk.try_into().unwrap();
            if *punch_chunk == [0xEE, 0xEE, 0xEE, 0xEE] {
                break;
            }
            punches.push(Punch::deserialize(punch_chunk)?);
        }

        return Ok(Self {
            card_personal_data2,
            punches,
        });
    }
}

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(export))]
#[derive(Debug)]
pub struct Card8Def {
    block0: Option<Block0>,
    block1: Option<Block1>,
}

impl CardDefinition for Card8Def {
    const HAS_CARD_EXCLUSIVES: bool = false;
    type CardExclusivesType = ();

    fn new_empty() -> Self {
        Self {
            block0: None,
            block1: None,
        }
    }

    fn has_block(&self, block_id: u8) -> bool {
        match block_id {
            0 => self.block0.is_some(),
            1 => self.block1.is_some(),
            _ => false,
        }
    }

    fn get_personal_data(
        &self,
    ) -> Option<Result<CardPersonalData, DeserializeCardPersonalDataError>> {
        let block0 = &self.block0.as_ref()?;
        let mut personal_data_buffer: Vec<u8> = Vec::new();
        personal_data_buffer.extend_from_slice(&block0.card_personal_data1);

        if !block0.personal_data_finished {
            let block1 = &self.block1.as_ref()?;
            personal_data_buffer.extend_from_slice(&block1.card_personal_data2);
        }

        return Some(CardPersonalData::deserialize(&personal_data_buffer));
    }

    fn get_punches(&self) -> Option<Vec<Punch>> {
        let block1 = &self.block1.as_ref()?;

        Some(block1.punches.clone())
    }

    fn get_punch_count(&self) -> Option<u8> {
        Some(self.block0.as_ref()?.punch_count)
    }

    fn get_start(&self) -> Option<Option<Punch>> {
        Some(self.block0.as_ref()?.start)
    }

    fn get_siid(&self) -> Option<u32> {
        Some(self.block0.as_ref()?.siid)
    }

    fn get_clear_check(&self) -> Option<Punch> {
        Some(self.block0.as_ref()?.clear_check)
    }

    fn get_finish(&self) -> Option<Option<Punch>> {
        Some(self.block0.as_ref()?.finish)
    }

    fn feed_block(
        &mut self,
        block_id: u8,
        block_buffer: &[u8; 128],
    ) -> Result<(), crate::errors::FeedBlockError> {
        match block_id {
            0 => {
                self.block0 = Some(Block0::deserialize(*block_buffer)?);
            }
            1 => {
                self.block1 = Some(Block1::deserialize(*block_buffer)?);
            }
            _ => return Err(crate::errors::FeedBlockError::BlockDoesNotExist),
        }
        return Ok(());
    }

    fn block_needed(&self, intention: &super::BlockNeededIntention) -> super::BlockNeededResult {
        match intention {
            BlockNeededIntention::CardExclusives => BlockNeededResult::NoNeed,
            BlockNeededIntention::CardPersonalData => {
                if let Some(block0) = &self.block0 {
                    if !block0.personal_data_finished && self.block1.is_none() {
                        return BlockNeededResult::Need(1);
                    } else {
                        return BlockNeededResult::NoNeed;
                    }
                } else {
                    return BlockNeededResult::Need(0);
                }
            }
            BlockNeededIntention::Punches => match &self.block1 {
                Some(_) => BlockNeededResult::NoNeed,
                None => BlockNeededResult::Need(1),
            },
        }
    }
}
