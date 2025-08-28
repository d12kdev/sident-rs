use chrono::NaiveDate;

// TODO: Docs
use crate::{
    addr_len::presets::SystemConfigAddrLen, errors::MakeSystemConfigError,
    firmware::FirmwareVersion, product::ProductModel, time::SIDate,
};

/// System config of the station
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(export))]
#[derive(Debug)]
pub struct SystemConfig {
    pub serial: u32,
    pub srr_config: u8, // TODO: Bitflags for SRRConfig
    pub firmware_ver: FirmwareVersion,
    pub produced: NaiveDate,
    pub model: ProductModel,
    pub memory_kb: u8,
    pub last_modification: NaiveDate, // TODO: other fields
}

impl SystemConfig {
    /// Deserializes SystemConfig from data
    pub fn deserialize(input: &[u8; 128]) -> Result<Self, MakeSystemConfigError> {
        // SRR CONFIG
        let srr_config = input[SystemConfigAddrLen::srr_config()][0];
        // FIRMWARE VERSION
        let firmware_ver = FirmwareVersion::deserialize(
            input[SystemConfigAddrLen::firmware_version()].try_into()?,
        );
        // PRODUCED
        let produced = SIDate::deserialize(input[SystemConfigAddrLen::prod_date()].try_into()?)
            .ok_or(MakeSystemConfigError::Other(format!(
                "could not get the produced date"
            )))?;
        // MODEL
        let _model_num =
            u16::from_be_bytes(input[SystemConfigAddrLen::product_model()].try_into()?);
        let model = ProductModel::try_from(_model_num)?;

        // SERIAL NUMBER
        let serial: u32 = match model {
            ProductModel::SimSrr => {
                let data: [u8; 3] =
                    input[SystemConfigAddrLen::simsrr_serial_number()].try_into()?;
                u32::from_be_bytes([data[0], data[1], data[2], 0])
            }
            _ => u32::from_be_bytes(input[SystemConfigAddrLen::serial_number()].try_into()?),
        };

        // MEMORY KBs
        let memory_kb = input[SystemConfigAddrLen::memory_kb()][0];
        // WAKEUP DATE
        let last_modification =
            SIDate::deserialize(input[SystemConfigAddrLen::last_modification()].try_into()?)
                .ok_or(MakeSystemConfigError::Other(format!(
                    "could not get the wakeup date"
                )))?;

        return Ok(Self {
            serial,
            srr_config,
            firmware_ver,
            produced,
            model,
            memory_kb,
            last_modification,
        });
    }
}

/// SRR channel
/// Source: SPORTident.Communication.SimSrrFrequencyChannels
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum SrrChannel {
    Red = 0x00,
    Blue = 0x01,
}

/// Operating mode of the station
/// Source: SPORTident.Communication.OperatingMode
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum OperatingMode {
    DControl = 0x01,
    Control = 0x02,
    Start = 0x03,
    Finish = 0x04,
    Readout = 0x05,
    Clear = 0x07,
    Check = 0x0A,
    Printout = 0x0B,
    StartWithTime = 0x0C,
    FinishWithTime = 0x0D,
    BcDControl = 0x11,
    BcControl = 0x12,
    BcStart = 0x13,
    BcFinish = 0x14,
    BcCheck = 0x1A,
    BcLineMasSta = 0x1C,
    BcLineMasFin = 0x1D,
    BcLineSlave1 = 0x1E,
    BcLineSlave2 = 0x1F,
}
