use crate::errors::FirmwareVersionCodecError;

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(export))]
#[derive(Debug, Default)]
pub struct FirmwareVersion {
    inner: [u8; 3],
}

impl FirmwareVersion {
    pub fn deserialize(input: &[u8; 3]) -> Self {
        Self { inner: *input }
    }

    pub fn to_string(&self) -> Result<String, FirmwareVersionCodecError> {
        return Ok(String::from_utf8(self.inner.to_vec())?);
    }

    pub fn to_u32(&self) -> Result<u32, FirmwareVersionCodecError> {
        return Ok(self.to_string()?.parse()?);
    }
}
