use chrono::{Datelike, NaiveDate};
use num_enum::TryFromPrimitive;

/// Impl of the way SPORTident stores dates
pub struct SIDate;

impl SIDate {
    pub fn from_bytes(input: [u8; 3]) -> Option<NaiveDate> {
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

#[derive(Debug, Clone, Copy, TryFromPrimitive)]
#[repr(u8)]
pub enum DayOfWeek {
    Monday = 0,
    Tuesday = 1,
    Wednesday = 2,
    Thursday = 3,
    Friday = 4,
    Saturday = 5,
    Sunday = 6,
}

#[derive(Debug, Clone, Copy, TryFromPrimitive)]
#[repr(u8)]
pub enum WeekOfMonth {
    Week1 = 0,
    Week2 = 1,
    Week3 = 2,
    Week4 = 3,
}