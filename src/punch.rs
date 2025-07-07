use bitflags::bitflags;
use chrono::{Duration, NaiveTime};

use crate::{card::CardType, errors::DeserializePunchError, time::{DayOfWeek, WeekOfMonth}};

#[derive(Debug, Clone, Copy)]
pub struct PunchTime {
    pub time: NaiveTime,
    pub day_of_week: DayOfWeek,
    pub week: WeekOfMonth
}

#[derive(Debug, Clone, Copy)]
pub struct Punch {
    pub station_code: u8,
    pub punch_time: PunchTime
}

impl Punch {
    pub fn deserialize(data: [u8; 4], card_type: CardType) -> Result<Self, DeserializePunchError> {
        match card_type {
            CardType::Card8 |
            CardType::Card9 |
            CardType::Card10 |
            CardType::Card11 |
            CardType::ActiveCard |
            CardType::PCard => {
                let station_code = data[1];
                let secs = u16::from_be_bytes([data[2], data[3]]);
                // TODO: some checks
                let mut time = NaiveTime::from_num_seconds_from_midnight_opt(secs as u32, 0)
                    .ok_or(DeserializePunchError::InvalidTime)?;
                let punch_flags = PunchFlags::from_bits_truncate(data[0]);

                if punch_flags.contains(PunchFlags::OFFSET_12H) {
                    time.overflowing_add_signed(Duration::hours(12));
                }

                let day_bits = (data[0] & PunchFlags::DAY_OF_WEEK.bits()) >> 1;
                let week_bits = (data[0] & PunchFlags::WEEK.bits()) >> 4;
                let day_of_week = match (day_bits.wrapping_sub(1)) % 7 {
                    0 => DayOfWeek::Monday,
                    1 => DayOfWeek::Tuesday,
                    2 => DayOfWeek::Wednesday,
                    3 => DayOfWeek::Thursday,
                    4 => DayOfWeek::Friday,
                    5 => DayOfWeek::Saturday,
                    6 => DayOfWeek::Sunday,
                    _ => return Err(DeserializePunchError::InvalidDay)
                };

                let week = match week_bits {
                    0 => WeekOfMonth::Week1,
                    1 => WeekOfMonth::Week2,
                    2 => WeekOfMonth::Week3,
                    3 => WeekOfMonth::Week4,
                    _ => return Err(DeserializePunchError::InvalidWeek)
                };

                Ok(Self {
                    station_code,
                    punch_time: PunchTime { time, day_of_week, week }
                })
            },
            _ => unimplemented!()
        }
    }
}

bitflags! {
    pub struct PunchFlags: u8 {
        const OFFSET_12H     = 0b0000_0001;
        const DAY_OF_WEEK    = 0b0000_1110;
        const WEEK   = 0b0011_0000;
    }
}