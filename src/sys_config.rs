use chrono::NaiveDate;

// TODO: Docs
use crate::{errors::MakeSystemConfigError, firmware::FirmwareVersion, product::ProductModel, time::SIDate, DataAddressAndLength};

#[derive(Debug, Default)]
pub struct SystemConfig {
    pub serial: u32,
    pub srr_config: u8, // TODO: Bitflags for SRRConfig
    pub firmware_ver: FirmwareVersion,
    pub produced: NaiveDate,
    pub model: ProductModel,
    pub memory_kb: u8,
    pub last_modification: NaiveDate
    // TODO: other fields
}

impl SystemConfig {
    pub fn from_bytes(input: [u8; 128]) -> Result<Self, MakeSystemConfigError> {
        // SERIAL
        let _o = DataAddressAndLength::SerialNumber.offset() as usize;
        let serial: u32 = u32::from_be_bytes(input[_o.._o + 4].try_into()?);
        // SRR CONFIG
        let _o = DataAddressAndLength::SrrConfig.offset() as usize;
        let srr_config = input[_o];
        // FIRMWARE VERSION
        let _o = DataAddressAndLength::FirmwareVersion.offset() as usize;
        let firmware_ver = FirmwareVersion::from_bytes(input[_o.._o + 3].try_into()?);
        // PRODUCED
        let _o = DataAddressAndLength::ProducedDate.offset() as usize;
        let produced = SIDate::from_bytes(input[_o.._o + 3].try_into()?).ok_or(MakeSystemConfigError::Other(format!("could not get the produced date")))?;
        // MODEL
        let _o = DataAddressAndLength::ProductModel.offset() as usize;
        let _model_num = u16::from_be_bytes(input[_o.._o + 2].try_into()?);
        let model = ProductModel::try_from(_model_num)?;
        // MEMORY KBs
        let _o = DataAddressAndLength::MemoryKb.offset() as usize;
        let memory_kb = input[_o];
        // WAKEUP DATE
        let _o = DataAddressAndLength::LastModification.offset() as usize;
        let last_modification = SIDate::from_bytes(input[_o.._o + 3].try_into()?).ok_or(MakeSystemConfigError::Other(format!("could not get the wakeup date")))?;

        return Ok(
            Self {
                serial,
                srr_config,
                firmware_ver,
                produced,
                model,
                memory_kb,
                last_modification
            }
        )
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
