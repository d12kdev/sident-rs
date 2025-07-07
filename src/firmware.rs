use crate::errors::FirmwareVersionCodecError;

#[derive(Debug, Default)]
pub struct FirmwareVersion {
    inner: [u8; 3],
}

impl FirmwareVersion {
    pub fn from_bytes(input: [u8; 3]) -> Self {
        Self { inner: input }
    }

    pub fn to_string(&self) -> Result<String, FirmwareVersionCodecError> {
        return Ok(String::from_utf8(self.inner.to_vec())?);
    }

    pub fn to_u32(&self) -> Result<u32, FirmwareVersionCodecError> {
        return Ok(self.to_string()?.parse()?);
    }
}