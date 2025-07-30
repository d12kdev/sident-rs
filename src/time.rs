use chrono::{Datelike, NaiveDate};
use num_enum::TryFromPrimitive;

/// Impl of the way SPORTident stores dates
pub struct SIDate;

impl SIDate {
    pub fn deserialize(input: [u8; 3]) -> Option<NaiveDate> {
        let year = (input[0] as i32) + 2000;
        let month = input[1] as u32;
        let day = input[2] as u32;

        NaiveDate::from_ymd_opt(year, month, day)
    }

    pub fn to_bytes(input: NaiveDate) -> Vec<u8> {
        let mut buffer = Vec::new();
        buffer.push((input.year() - 2000) as u8);
        buffer.push(input.month() as u8);
        buffer.push(input.day() as u8);

        return buffer;
    }
}

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(export))]
#[derive(Debug, Clone, Copy, TryFromPrimitive, Default)]
#[repr(u8)]
/// Source: SI_cards_data_structure_developer.ods
pub enum DayOfWeek {
    Monday = 1,
    Tuesday = 2,
    Wednesday = 3,
    Thursday = 4,
    Friday = 5,
    Saturday = 6,
    #[default]
    Sunday = 0,
}

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(export))]
#[derive(Debug, Clone, Copy, TryFromPrimitive, Default)]
#[repr(u8)]
pub enum WeekOfMonth {
    #[default]
    Week1 = 0,
    Week2 = 1,
    Week3 = 2,
    Week4 = 3,
}
