use bitflags::bitflags;
use chrono::{NaiveTime, Timelike};
use log::error;
use num_enum::TryFromPrimitive;

use crate::{
    errors::DeserializePunchError,
    time::{DayOfWeek, WeekOfMonth},
};

/// Timing part of the punch
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(export))]
#[derive(Debug, Clone, Copy, Default)]
pub struct PunchTime {
    pub time: NaiveTime,
    pub day: DayOfWeek,
    pub week: WeekOfMonth,
}

impl PunchTime {
    /// Converts the time,day,week timing to absolute seconds
    pub fn to_absolute_seconds(&self) -> i64 {
        let week = self.week as i64;
        let day = self.day as i64;

        let days_in_total = week * 7 + day;
        let seconds_in_day = self.time.num_seconds_from_midnight() as i64;

        days_in_total * 86_400 + seconds_in_day
    }

    /// Duration since other Punch
    ///
    /// * `other` - Other punch
    pub fn duration_since(&self, other: &Self) -> chrono::Duration {
        let self_abs = self.to_absolute_seconds();
        let other_abs = other.to_absolute_seconds();
        chrono::Duration::seconds(self_abs - other_abs)
    }
}

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(export))]
#[derive(Debug, Clone, Copy, Default)]
pub struct Punch {
    pub station_code: u16,
    pub punch_time: PunchTime,
}

impl Punch {
    /// Checks if data is null and deserializes a Punch
    ///
    /// * `data` - Punch data
    pub fn deserialize_control_punch(
        data: &[u8; 4],
    ) -> Result<Option<Self>, DeserializePunchError> {
        if *data == [0xEE, 0xEE, 0xEE, 0xEE] {
            return Ok(None);
        }

        return Ok(Some(Self::deserialize(data)?));
    }

    /// Deserializes a Punch
    ///
    /// * `data` - Punch data
    ///
    /// Source: SPORTident.Communication.Communication._parseMemoryRecord
    // TODO: look at the Communication.AllowCardReadSubSeconds thing
    pub fn deserialize(data: &[u8; 4]) -> Result<Self, DeserializePunchError> {
        let td_byte = data[0];
        let cn_byte = data[1];
        let th_byte = data[2];
        let tl_byte = data[3];

        if td_byte == 0xFF && cn_byte == 0xFF && th_byte == 0xFF && tl_byte == 0xFF {
            return Err(DeserializePunchError::DataCleared);
        };

        let day_of_week = (td_byte & PunchFlags::DAY_OF_WEEK.bits()) >> 1;
        let day_of_week = DayOfWeek::try_from_primitive(day_of_week).map_err(|e| {
            error!("{}", e.to_string());
            return DeserializePunchError::InvalidDay;
        })?;

        let high = (td_byte & 0b1100_0000) >> 6; // bits 6–7 → bits 0–1
        let station_code = ((high as u16) << 8) | (cn_byte as u16);

        let time_punched_seconds = u16::from_be_bytes([th_byte, tl_byte]);
        let mut time_punched =
            NaiveTime::from_num_seconds_from_midnight_opt(time_punched_seconds as u32, 0)
                .ok_or(DeserializePunchError::InvalidTime)?;

        if (td_byte & PunchFlags::OFFSET_12H.bits()) == 1 {
            time_punched += chrono::Duration::hours(12);
        }

        let week_of_month = (td_byte & PunchFlags::WEEK.bits()) >> 4;
        let week_of_month = WeekOfMonth::try_from_primitive(week_of_month)
            .map_err(|_| return DeserializePunchError::InvalidWeek)?;

        return Ok(Self {
            station_code,
            punch_time: PunchTime {
                time: time_punched,
                day: day_of_week,
                week: week_of_month,
            },
        });
    }
}

bitflags! {
    pub struct PunchFlags: u8 {
        const OFFSET_12H =  0b0000_0001;
        const DAY_OF_WEEK = 0b0000_1110;
        const WEEK   =      0b0011_0000;
    }
}
