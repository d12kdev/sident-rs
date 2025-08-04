/*
    SPORTident Card-9 Memory Structure

    Every integer is big endian encoded unless said differently in the desc.

    ---- BLOCK 0x00 ----
    0x00..0x03  Unique device ID
    0x04..0x07  Start of record - 0xEA 0xEA 0xEA 0xEA
    0x08..0x0B  Clear/Check (punch)
    0x0C..0x0F  Start (punch)
    0x10..0x13  Finish (punch)
    0x14..0x15  Last visited control
    0x16        Punch pointer - punch count
    0x18..0x1B  SIID - SI3, SI2, SI1, SI0 - SI2..SI0 makes up the SIID (card number)
    0x1C        Prod date MONTH
    0x1D        Prod date YEAR (2000+x)
    0x20..0x37  Card personal data (Only first and last name)
    0x38..0x7F  Punches
    ----            ----
    ---- BLOCK 0x01 ----
    0x00..0x7F  Punches
    ----            ----
*/

use crate::{
    card::CardPersonalData,
    carddef::{BlockNeededIntention, BlockNeededResult, CardDefinition},
    errors::DeserializeBlockError,
    extract_fixed,
    punch::Punch,
};

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(export, export_to = "SI9Block0.ts"))]
#[derive(Debug, Clone)]
struct Block0 {
    uid: u32,
    clear_check: Punch,
    start: Option<Punch>,
    finish: Option<Punch>,
    last_visited: u16,
    punch_count: u8,
    siid: u32,
    prod_month: u8,
    prod_year: u8,
    card_personal_data: [u8; 24],
    punches: Vec<Punch>,
    punches_finished: bool,
}

impl Block0 {
    pub fn deserialize(data: [u8; 128]) -> Result<Self, DeserializeBlockError> {
        let uid = u32::from_be_bytes(extract_fixed!(&data, 0x00..0x03));
        let clear_check = Punch::deserialize(&extract_fixed!(&data, 0x08..0x0B))?;
        let start = Punch::deserialize_control_punch(&extract_fixed!(&data, 0x0C..0x0F))?;
        let finish = Punch::deserialize_control_punch(&extract_fixed!(&data, 0x10..0x13))?;
        let last_visited = u16::from_be_bytes(extract_fixed!(&data, 0x14..0x15));
        let punch_count = data[0x16];
        let mut siid_bytes = extract_fixed!(&data, 0x18..0x1B);
        siid_bytes[0] = 0;
        let siid = u32::from_be_bytes(siid_bytes);
        let prod_month = data[0x1C];
        let prod_year = data[0x1D];
        let card_personal_data = extract_fixed!(&data, 0x20..0x37);

        let mut punches_finished = false;
        let mut punches: Vec<Punch> = Vec::new();
        let punches_bytes = extract_fixed!(&data, 0x38..0x7F);

        for punch_chunk in punches_bytes.chunks(4) {
            let punch_chunk: &[u8; 4] = punch_chunk.try_into().unwrap(); // cant fail
            // if its 0xEE 0xEE 0xEE 0xEE then there should be no more data left
            if punch_chunk.iter().all(|&b| b == 0xEE) {
                punches_finished = true;
                break;
            }

            punches.push(Punch::deserialize(punch_chunk)?);
        }

        return Ok(Self {
            uid,
            clear_check,
            start,
            finish,
            last_visited,
            punch_count,
            siid,
            prod_month,
            prod_year,
            card_personal_data,
            punches,
            punches_finished,
        });
    }
}

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(export, export_to = "SI9Block1.ts"))]
#[derive(Debug, Clone)]
struct Block1 {
    punches: Vec<Punch>,
}

impl Block1 {
    pub fn deserialize(data: [u8; 128]) -> Result<Self, DeserializeBlockError> {
        let mut punches: Vec<Punch> = Vec::new();

        for punch_chunk in data.chunks(4) {
            let punch_chunk: &[u8; 4] = punch_chunk.try_into().unwrap(); // cant fail
            // if its 0xEE 0xEE 0xEE 0xEE then there should be no more data left
            if punch_chunk.iter().all(|&b| b == 0xEE) {
                break;
            }

            punches.push(Punch::deserialize(punch_chunk)?);
        }

        return Ok(Self { punches });
    }
}

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(export))]
#[derive(Debug, Clone)]
pub struct Card9Def {
    block0: Option<Block0>,
    block1: Option<Block1>,
}

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(export))]
#[derive(Debug, Clone)]
pub struct Card9Exclusives {
    pub uid: u32,
    pub last_visited_station_code: u16,
    pub production_date_month: u8,
    pub production_date_year: u32,
}

impl CardDefinition for Card9Def {
    const HAS_CARD_EXCLUSIVES: bool = true;
    type CardExclusivesType = Card9Exclusives;
    fn get_exclusives(&self) -> Option<Self::CardExclusivesType> {
        let block0 = self.block0.as_ref()?;

        return Some(Self::CardExclusivesType {
            uid: block0.uid,
            last_visited_station_code: block0.last_visited,
            production_date_month: block0.prod_month,
            production_date_year: 2000 + block0.prod_year as u32,
        });
    }

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
    ) -> Option<
        Result<crate::card::CardPersonalData, crate::errors::DeserializeCardPersonalDataError>,
    > {
        let block0 = self.block0.as_ref()?;
        return Some(CardPersonalData::deserialize(&block0.card_personal_data));
    }

    fn get_punches(&self) -> Option<Vec<Punch>> {
        let block0 = self.block0.as_ref()?;

        if block0.punches_finished {
            return Some(block0.punches.clone());
        }

        let block1 = self.block1.as_ref()?;

        let mut punches: Vec<Punch> = Vec::new();
        punches.extend_from_slice(&block0.punches.clone());
        punches.extend_from_slice(&block1.punches.clone());

        return Some(punches);
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
            BlockNeededIntention::CardExclusives | BlockNeededIntention::CardPersonalData => {
                match &self.block0 {
                    Some(_) => BlockNeededResult::NoNeed,
                    None => BlockNeededResult::Need(0),
                }
            }
            BlockNeededIntention::Punches => {
                if let Some(block0) = &self.block0 {
                    if !block0.punches_finished && self.block1.is_none() {
                        return BlockNeededResult::Need(1);
                    }
                    return BlockNeededResult::NoNeed;
                }
                return BlockNeededResult::Need(0);
            }
        }
    }
}
