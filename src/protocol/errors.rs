use thiserror::Error;

#[derive(Debug, Error)]
pub enum DecoderError {
    #[error("Wrong start byte ({0})")]
    WrongStartByte(u8),
    #[error("Wrong end byte ({0})")]
    WrongEndByte(u8),
    #[error("Input buffer is too short")]
    InputTooShort,
    #[error("Input buffer is longer than expected")]
    InputTooLong,
    #[error("CRC does not match")]
    CrcDoesNotMatch,
}
