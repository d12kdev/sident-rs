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
    #[error("Legacy/base protocol is not supported")]
    LegacyNotSupported,
}

#[derive(Debug, Error)]
pub enum DecodePacketError {
    #[error("Wrong packet id")]
    WrongPacketId,
    #[error("Input buffer is too short")]
    InputTooShort,
    #[error("Input response is NAK")]
    InputIsNak,
    #[error("Input buffer doesn't seem like matching the packet")]
    WrongData,
}
