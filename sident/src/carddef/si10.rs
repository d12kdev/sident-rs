/*
    SPORTident Card-10 Memory Structure - Very similar to SIAC, but SI10/11 does not have battery

    Every integer is big endian encoded unless said differently in the desc.

    ---- BLOCK 0x00 ----
    0x00..0x03  Unique device ID
    0x04..0x07  Start of record - 0xEA 0xEA 0xEA 0xEA
    0x08..0x0B  Clear/Check (punch)
    0x0C..0x0F  Start (punch)
    0x10..0x13  Finish (punch)
    0x14        Reserve pointer
    0x15        Undocumented CRC8
    0x16        Punch pointer - punch count
    0x18..0x1B  SIID - SI3, SI2, SI1, SI0 - SI2..SI0 makes up the SIID (card number)
    0x1C        Production month
    0x1D        Production year (2000 + year)
    0x1E..0x1F  Undocumented CRC
    0x20..0x7F  Card personal data - PART1
    ----            ----
    ---- BLOCK 0x01 ----
    0x00..0x1F  Card personal data - PART2
    0x20..0x2B  Nothing
    0x2C..0x2F  Block 1 identifier - 0xEB 0xEB 0xEB 0xEB
    0x30..0x7F  Reserve (nothing)
    ----            ----
    ---- BLOCK 0x02 ---- (aka most useful block)
    0x00..0x7B  Reserve (nothing)
    0x7C..0x7F  Block 2 identifier - 0xEC 0xEC 0xEC 0xEC
    ----            ----
    ---- BLOCK 0x03 ----
    0x38..0x3B  Clear/Check reserve (punch)
    0x3C..0x3E  Probably production date, BYTE0=YY BYTE1=MM BYTE2=DD
    0x3F        Character set used
    0x40..0x41  Hardware version
    0x42..0x43  Software version
    0x48..0x49  Usage/Clear count
    0x58..0x5B  Start reserve (punch)
    0x5C..0x5F  Finish reserve (punch)
    0x70..0x73  SISYS?? idk but its always 0x73,0x69,0x61,0x63 and in HxD it says "siac"
    0x74..0x7B  Trim values?? what is this
    0x7C..0x7F  Device configuration - idk what each byte means tho
    ----            ----
    ---- BLOCK 4..7 ----
    Every 4 bytes is a punch - if all 4 bytes are 0xEE then you can stop parsing because there will be no more punches - but you have the punch count to be safe.
*/

use chrono::NaiveDate;

use crate::{
    card::CardPersonalData,
    carddef::{BlockNeededIntention, BlockNeededResult, CardDefinition},
    errors::{DeserializeBlockError, DeserializeCardPersonalDataError},
    extract_fixed,
    punch::Punch,
};

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(export, export_to = "SI10Block0.ts"))]
#[derive(Debug, Clone)]
struct Block0 {
    uid: u32,
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
        let uid_bytes = extract_fixed!(&data, 0x00..0x03);
        let uid = u32::from_be_bytes(uid_bytes);

        let clear_check_bytes = extract_fixed!(&data, 0x08..0x0B);
        let clear_check = Punch::deserialize(&clear_check_bytes)?;

        let start_bytes = extract_fixed!(&data, 0x0C..0x0F);
        let start = Punch::deserialize_control_punch(&start_bytes)?;

        let finish_bytes = extract_fixed!(&data, 0x10..0x13);
        let finish = Punch::deserialize_control_punch(&finish_bytes)?;

        let punch_count = data[0x16];

        let mut siid_bytes = extract_fixed!(&data, 0x18..0x1B);
        siid_bytes[0] = 0x00;
        let siid = u32::from_be_bytes(siid_bytes);

        let card_personal_data1 = extract_fixed!(&data, 0x20..0x7F);

        let personal_data_finished = [data[126], data[127]] == [0xEE, 0xEE];

        return Ok(Self {
            uid,
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
#[cfg_attr(feature = "ts-rs", ts(export, export_to = "SI10Block1.ts"))]
#[derive(Debug)]
struct Block1 {
    card_personal_data2: [u8; 32],
}

impl Block1 {
    pub fn deserialize(data: [u8; 128]) -> Result<Self, DeserializeBlockError> {
        let card_personal_data2 = extract_fixed!(&data, 0x00..0x1F);

        return Ok(Self {
            card_personal_data2,
        });
    }
}

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(export, export_to = "SI10Block3.ts"))]
#[derive(Debug)]
struct Block3 {
    clear_check_reserve: Punch,
    prod_date: NaiveDate,
    hw_version: u16,
    sw_version: u16,
    clear_count: u16,
    start_reserve: Option<Punch>,
    finish_reserve: Option<Punch>,
}

impl Block3 {
    pub fn deserialize(data: [u8; 128]) -> Result<Self, DeserializeBlockError> {
        let clear_check_reserve_bytes = extract_fixed!(&data, 0x38..0x3B);
        let clear_check_reserve = Punch::deserialize(&clear_check_reserve_bytes)?;

        let prod_date_bytes = extract_fixed!(&data, 0x3C..0x3E);
        let prod_date = NaiveDate::from_ymd_opt(
            (prod_date_bytes[0] as i32) + 2000,
            prod_date_bytes[1] as u32,
            prod_date_bytes[2] as u32,
        )
        .ok_or(DeserializeBlockError::DateError)?;

        let hw_version_bytes = extract_fixed!(&data, 0x40..0x41);
        let hw_version = u16::from_be_bytes(hw_version_bytes);

        let sw_version_bytes = extract_fixed!(&data, 0x42..0x43);
        let sw_version = u16::from_be_bytes(sw_version_bytes);

        let clear_count_bytes = extract_fixed!(&data, 0x48..0x49);
        let clear_count = u16::from_be_bytes(clear_count_bytes);

        let start_reserve_bytes = extract_fixed!(&data, 0x58..0x5B);
        let start_reserve = Punch::deserialize_control_punch(&start_reserve_bytes)?;

        let finish_reserve_bytes = extract_fixed!(&data, 0x5C..0x5F);
        let finish_reserve = Punch::deserialize_control_punch(&finish_reserve_bytes)?;

        return Ok(Self {
            clear_check_reserve,
            prod_date,
            hw_version,
            sw_version,
            clear_count,
            start_reserve,
            finish_reserve,
        });
    }
}

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(export, export_to = "SI10Block4.ts"))]
#[derive(Debug)]
struct Block4 {
    punches: Vec<Punch>,
    punches_finished: bool,
}

impl Block4 {
    pub fn deserialize(data: [u8; 128]) -> Result<Self, DeserializeBlockError> {
        let mut punches: Vec<Punch> = Vec::new();
        let mut punches_finished = false;

        for punch_chunk in data.chunks(4) {
            let punch_chunk: &[u8; 4] = punch_chunk.try_into().unwrap(); // cant fail
            // if its 0xEE 0xEE 0xEE 0xEE then there should be no more data left
            if punch_chunk.iter().all(|&b| b == 0xEE) {
                punches_finished = true;
                break;
            }

            punches.push(Punch::deserialize(punch_chunk)?);
        }

        return Ok(Self {
            punches,
            punches_finished,
        });
    }
}

type Block5 = Block4;
type Block6 = Block4;
type Block7 = Block4;

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(export))]
#[derive(Debug)]
pub struct Card10Def {
    block0: Option<Block0>,
    block1: Option<Block1>,
    // block 2 is not useful,
    block3: Option<Block3>,
    block4: Option<Block4>,
    block5: Option<Block5>,
    block6: Option<Block6>,
    block7: Option<Block7>,
}

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(export))]
#[derive(Debug)]
pub struct Card10Exclusives {
    pub uid: u32,
    pub clear_check_reserve: Punch,
    pub prod_date: NaiveDate,
    pub hw_version: u16,
    pub sw_version: u16,
    pub clear_count: u16,
    pub start_reserve: Option<Punch>,
    pub finish_reserve: Option<Punch>,
}

impl CardDefinition for Card10Def {
    const HAS_CARD_EXCLUSIVES: bool = true;
    type CardExclusivesType = Card10Exclusives;
    fn get_exclusives(&self) -> Option<Self::CardExclusivesType> {
        let block0 = self.block0.as_ref()?;
        let block3 = self.block3.as_ref()?;

        return Some(Self::CardExclusivesType {
            uid: block0.uid,
            clear_check_reserve: block3.clear_check_reserve,
            prod_date: block3.prod_date,
            hw_version: block3.hw_version,
            sw_version: block3.sw_version,
            clear_count: block3.clear_count,
            start_reserve: block3.start_reserve,
            finish_reserve: block3.finish_reserve,
        });
    }

    fn new_empty() -> Self {
        Self {
            block0: None,
            block1: None,
            block3: None,
            block4: None,
            block5: None,
            block6: None,
            block7: None,
        }
    }

    fn get_punches(&self) -> Option<Vec<Punch>> {
        if self.block_needed(&BlockNeededIntention::Punches) != BlockNeededResult::NoNeed {
            return None;
        }

        let block4 = self.block4.as_ref()?;

        let mut final_punches = Vec::new();

        // Build a slice of the blocks
        let blocks: [Option<&Block4>; 4] = [
            Some(block4),
            self.block5.as_ref(),
            self.block6.as_ref(),
            self.block7.as_ref(),
        ];

        // Process blocks in order until punches_finished
        for block in blocks.iter().flatten() {
            final_punches.extend_from_slice(&block.punches);
            if block.punches_finished {
                break;
            }
        }

        Some(final_punches)
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

    fn get_punch_count(&self) -> Option<u8> {
        Some(self.block0.as_ref()?.punch_count)
    }

    fn get_start(&self) -> Option<Option<Punch>> {
        Some(self.block0.as_ref()?.start)
    }

    fn block_needed(&self, intention: &super::BlockNeededIntention) -> super::BlockNeededResult {
        match intention {
            BlockNeededIntention::CardExclusives => {
                match self.block0 {
                    Some(_) => {}
                    None => return BlockNeededResult::Need(0),
                }

                match self.block3 {
                    Some(_) => {}
                    None => return BlockNeededResult::Need(3),
                }

                return BlockNeededResult::NoNeed;
            }
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
            BlockNeededIntention::Punches => {
                for (block_id, block) in [
                    (4, &self.block4),
                    (5, &self.block5),
                    (6, &self.block6),
                    (7, &self.block7),
                ] {
                    match block {
                        None => return BlockNeededResult::Need(block_id),
                        Some(block) => {
                            if !block.punches_finished {
                                return BlockNeededResult::Need(block_id + 1);
                            } else {
                                return BlockNeededResult::NoNeed;
                            }
                        }
                    }
                }

                return BlockNeededResult::NoNeed;
            }
        }
    }

    fn has_block(&self, block_id: u8) -> bool {
        match block_id {
            0 => self.block0.is_some(),
            1 => self.block1.is_some(),
            2 => true,
            3 => self.block3.is_some(),
            4 => self.block4.is_some(),
            5 => self.block5.is_some(),
            6 => self.block6.is_some(),
            7 => self.block7.is_some(),
            _ => false,
        }
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
            2 => (),
            3 => {
                self.block3 = Some(Block3::deserialize(*block_buffer)?);
            }
            4 => {
                self.block4 = Some(Block4::deserialize(*block_buffer)?);
            }
            5 => {
                self.block5 = Some(Block4::deserialize(*block_buffer)?);
            }
            6 => {
                self.block6 = Some(Block4::deserialize(*block_buffer)?);
            }
            7 => {
                self.block7 = Some(Block4::deserialize(*block_buffer)?);
            }
            _ => return Err(crate::errors::FeedBlockError::BlockDoesNotExist),
        }
        return Ok(());
    }
}
