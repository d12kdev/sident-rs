/*
    SPORTident Card-6 Memory Structure

    Every integer is big endian encoded unless said differently in the desc.

    ---- BLOCK 0x00 ----
    0x0B..0x0D  SIID - SI2 SI1 SI0 (u24)
    0x12        Punch pointer - punch count
    0x13        Punch pointer reserve
    0x14..0x17  Finish
    0x18..0x1B  Start
    0x1C..0x1F  Check
    0x20..0x23  Clear
    || card personal data - start
    0x28..0x2B  LITTLE ENDIAN! Start number
    0x30..0x43  Surname
    0x44..0x57  First name
    0x58..0x5B  Country
    0x5C..0x7F  Club
    ----            ----
    ---- BLOCK 0x01 ----
    0x00..0x0F  UID - idk what is this
    0x10..0x1F  Phone
    0x20..0x43  E-mail
    0x44..0x57  Street
    0x58..0x67  City
    0x68..0x6F  ZIP
    0x70..0x73  Sex/Gender
    0x74..0x7B  Date of birth
    || card personal data - end
    0x7C..0x7F  Date of production
    ----            ----
    ---- BLOCK 0x06 ----
    0x00..0x7F  Punches
    ----            ----
    ---- BLOCK 0x07 ----
    0x00..0x7F  Punches
    ----            ----


    --------------------
    SPORTident Card-6 Punch Format
    Used in: SI6, SI8, SI9, SI10, SI11, SIAC, pCard
    
    Structure: TD, CN, TH, TL (4 bytes)

    CN - Station code number LOWER
    TH, TL - 12h binary (seconds) [TH - Time Higher, TL - Time Lower]

    || TD byte:
    BIT-0       AM/PM (1 - AM, 0 - PM)
    BIT-3..1    Day of week
                001 (1) = Monday
                010 (2) = Tuesday
                011 (3) = Wednesday
                100 (4) = Thursday
                101 (5) = Friday
                110 (6) = Saturday
                000 (0) = Sunday
    BIT-5..4    Week of month
                00 (0) = Week 1
                01 (1) = Week 2
                10 (2) = Week 3
                11 (3) = Week 4
    BIT-7..6    Station code number HIGH
    ||
*/

use crate::{card::CardPersonalData, carddef::{BlockNeededIntention, BlockNeededResult, CardDefinition}, errors::DeserializeBlockError, extract_fixed, punch::Punch};

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(export, export_to = "SI6Block0.ts"))]
#[derive(Debug, Clone)]
struct Block0 {
    siid: u32,
    punch_count: u8,
    finish: Option<Punch>,
    start: Option<Punch>,
    check: Option<Punch>,
    clear: Punch,
    start_number: u32,
    // CARD PERSONAL DATA START
    #[cfg_attr(feature = "serde", serde(with = "serde_big_array::BigArray"))]
    #[cfg_attr(feature = "ts-rs", ts(type = "[number; 80]"))]
    card_personal_data1: [u8; 80],
    ctype_star: bool
}

impl Block0 {
    pub fn deserialize(data: [u8; 128]) -> Result<Self, DeserializeBlockError> {
        let siid = u32::from_be_bytes([0, data[0x0B], data[0x0C], data[0x0D]]);
        let punch_count = data[0x12];
        let finish = Punch::deserialize_control_punch(&extract_fixed!(&data, 0x14..0x17))?;
        let start = Punch::deserialize_control_punch(&extract_fixed!(&data, 0x18..0x1B))?;
        let check = Punch::deserialize_control_punch(&extract_fixed!(&data, 0x1C..0x1F))?;
        let clear = Punch::deserialize(&extract_fixed!(&data, 0x20..0x23))?;
        let start_number = u32::from_le_bytes(extract_fixed!(&data, 0x28..0x2B));
        let cpd1 = extract_fixed!(&data, 0x30..0x7F);
        let mut ctype_star = false;

        if punch_count > 64 {
            ctype_star = true;
        }

        return Ok(
            Self {
                siid,
                punch_count,
                finish,
                start,
                check,
                clear,
                start_number,
                card_personal_data1: cpd1,
                ctype_star
            }
        );
    }
}


#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(export, export_to = "SI6Block1.ts"))]
#[derive(Debug, Clone)]
struct Block1 {
    #[cfg_attr(feature = "serde", serde(with = "serde_big_array::BigArray"))]
    #[cfg_attr(feature = "ts-rs", ts(type = "[number; 124]"))]
    card_personal_data2: [u8; 124]
}

impl Block1 {
    pub fn deserialize(data: [u8; 128]) -> Result<Self, DeserializeBlockError> {
        let cpd2 = extract_fixed!(&data, 0x00..0x7B);

        return Ok(
            Self {
                card_personal_data2: cpd2
            }
        )
    }
}


#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(export, export_to = "SI6Block2.ts"))]
#[derive(Debug, Clone)]
/// SI6* - Block2 and higher, SI6 - Block6 and higher
struct Block2 {
    punches: Vec<Punch>,
    punches_finished: bool
}

impl Block2 {
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

        return Ok(Self { punches, punches_finished });
    }
}

type Block3 = Block2;
type Block4 = Block2;
type Block5 = Block2;
type Block6 = Block2;
type Block7 = Block2;

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(export))]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Card6Type {
    Regular,
    Star
}

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(export))]
#[derive(Debug, Clone)]
pub struct Card6Def {
    block0: Option<Block0>,
    block1: Option<Block1>,
    /// SI6*
    block2: Option<Block2>,
    /// SI6*
    block3: Option<Block3>,
    /// SI6*
    block4: Option<Block4>,
    /// SI6*
    block5: Option<Block5>,
    block6: Option<Block6>,
    block7: Option<Block7>,
    ctype: Card6Type,
    /// Are we sure about the ctype?
    ctype_ensured: bool
}

impl Card6Def {
    fn _get_blocks(&self) -> Vec<Option<&Block2>> {
        self._get_blocks_and_ids()
            .into_iter()
            .map(|(_, block)| block)
            .collect()
    }

    fn _get_blocks_and_ids(&self) -> Vec<(u8, Option<&Block2>)> {
        match self.ctype {
            Card6Type::Regular => {
                return vec![
                    (6, self.block6.as_ref()),
                    (7, self.block7.as_ref()),
                ];
            },
            Card6Type::Star => {
                return vec![
                    (2, self.block2.as_ref()),
                    (3, self.block3.as_ref()),
                    (4, self.block4.as_ref()),
                    (5, self.block5.as_ref()),
                    (6, self.block6.as_ref()),
                    (7, self.block7.as_ref()),
                ];
            }
        }
    }

    pub fn get_card6_type(&self) -> Card6Type {
        return self.ctype.clone();
    }
}

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(export))]
#[derive(Debug, Clone)]
pub struct Card6Exclusives {
    pub start_number: u32,
    pub card_type: Card6Type
}

impl CardDefinition for Card6Def {
    const HAS_CARD_EXCLUSIVES: bool = true;
    type CardExclusivesType = Card6Exclusives;
    fn get_exclusives(&self) -> Option<Self::CardExclusivesType> {
        let block0 = self.block0.as_ref()?;

        return Some(
            Self::CardExclusivesType {
                start_number: block0.start_number,
                card_type: self.ctype.clone()
            }
        );
    }

    fn new_empty() -> Self {
        Self {
            block0: None,
            block1: None,
            block2: None,
            block3: None,
            block4: None,
            block5: None,
            block6: None,
            block7: None,
            ctype: Card6Type::Regular,
            ctype_ensured: false,
        }
    }

    fn get_punches(&self) -> Option<Vec<Punch>> {
        let mut final_punches: Vec<Punch> = Vec::new();
        let blocks = self._get_blocks();

        for block in blocks {
            let block = block?;
            final_punches.extend_from_slice(&block.punches);
            if block.punches_finished {
                break;
            }
        }

        return Some(final_punches);
    }

    fn get_siid(&self) -> Option<u32> {
        Some(self.block0.as_ref()?.siid)
    }

    fn get_clear_check(&self) -> Option<Punch> {
        let block0 = self.block0.as_ref()?;
        if block0.check.is_some() {
            return block0.check;
        }
        return Some(block0.clear);
    }

    fn get_finish(&self) -> Option<Option<Punch>> {
        Some(self.block0.as_ref()?.finish)
    }

    fn get_personal_data(
            &self,
        ) -> Option<Result<crate::card::CardPersonalData, crate::errors::DeserializeCardPersonalDataError>> {
        let block0 = self.block0.as_ref()?;
        let block1 = self.block1.as_ref()?;
        let mut buff = Vec::new();
        buff.extend_from_slice(&block0.card_personal_data1);
        buff.extend_from_slice(&block1.card_personal_data2);
        let buff: [u8; 204] = buff.try_into().unwrap();
        return Some(CardPersonalData::deserialize_card_6(buff));
    }

    fn get_punch_count(&self) -> Option<u8> {
        Some(self.block0.as_ref()?.punch_count)
    }

    fn get_start(&self) -> Option<Option<Punch>> {
        Some(self.block0.as_ref()?.start)
    }

    fn block_needed(&self, intention: &BlockNeededIntention) -> BlockNeededResult {
        match intention {
            BlockNeededIntention::CardExclusives => {
                match self.block0 {
                    Some(_) => {},
                    None => return BlockNeededResult::Need(0),
                }

                if !self.ctype_ensured {
                    todo!()
                }
                return BlockNeededResult::NoNeed;
            },
            BlockNeededIntention::CardPersonalData => {
                match self.block0 {
                    Some(_) => {}
                    None => return BlockNeededResult::Need(0),
                }

                match self.block1 {
                    Some(_) => {}
                    None => return BlockNeededResult::Need(1),
                }

                return BlockNeededResult::NoNeed;
            },
            BlockNeededIntention::Punches => {
                for (block_id, block) in self._get_blocks_and_ids() {
                    let block = match block {
                        Some(ok) => ok,
                        None => return BlockNeededResult::Need(block_id)
                    };
                    if block.punches_finished {
                        break;
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
            2 => self.block2.is_some(),
            3 => self.block3.is_some(),
            4 => self.block4.is_some(),
            5 => self.block5.is_some(),
            6 => self.block6.is_some(),
            7 => self.block7.is_some(),
            _ => false,
        }
    }

    fn feed_block(&mut self, block_id: u8, block_buffer: &[u8; 128]) -> Result<(), crate::errors::FeedBlockError> {

        if self.ctype == Card6Type::Regular && (block_id < 6 && block_id > 1) {
            return Err(crate::errors::FeedBlockError::BlockDoesNotExist);
        }

        match block_id {
            0 => {
                let b0 = Block0::deserialize(*block_buffer)?;
                if b0.ctype_star {
                    self.ctype = Card6Type::Star;
                    self.ctype_ensured = true;
                }
                self.block0 = Some(b0);
            }
            1 => {
                self.block1 = Some(Block1::deserialize(*block_buffer)?);
            }
            2 => {
                self.block2 = Some(Block2::deserialize(*block_buffer)?);
            },
            3 => {
                self.block3 = Some(Block2::deserialize(*block_buffer)?);
            }
            4 => {
                self.block4 = Some(Block2::deserialize(*block_buffer)?);
            }
            5 => {
                self.block5 = Some(Block2::deserialize(*block_buffer)?);
            }
            6 => {
                self.block6 = Some(Block2::deserialize(*block_buffer)?);
            }
            7 => {
                self.block7 = Some(Block2::deserialize(*block_buffer)?);
            }
            _ => return Err(crate::errors::FeedBlockError::BlockDoesNotExist),
        }
        return Ok(());
    }
}