use std::{array::TryFromSliceError, num::ParseIntError};

use num_enum::TryFromPrimitiveError;
use thiserror::Error;

use crate::{card::CardType, product::ProductModel};

#[derive(Debug, Error)]
pub enum NewConnectionError {
    #[error("SerialPort error: {0}")]
    SerialportError(#[from] tokio_serial::Error),
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
    #[error("DeserializeRawPacket error: {0}")]
    DeserializeRawPacketError(#[from] DeserializeRawPacketError),
    #[error("DeserializePacket error: {0}")]
    DeserializePacketError(#[from] DeserializePacketError),
    #[error("MakeSystemConfig error: {0}")]
    MakeSystemConfigError(#[from] MakeSystemConfigError)
}

#[derive(Debug, Error)]
pub enum DeserializePacketError {
    #[error("receive raw packet error {0}")]
    DeserializeRawPacketError(#[from] DeserializeRawPacketError),
    #[error("response is NAK")]
    ResponseIsNak,
    #[error("data len is wrong")]
    WrongDataLen,
    #[error("{0}")]
    Other(String),
}

#[derive(Debug, Error)]
pub enum DeserializeRawPacketError {
    #[error("serial error {0}")]
    SerialportError(#[from] tokio_serial::Error),
    #[error("io error {0}")]
    IoError(#[from] std::io::Error),
    #[error("timed out")]
    TimedOut,
    #[error("parse error")]
    ParseError,
    #[error("crc is invalid")]
    CrcError,
}

#[derive(Debug, Error)]
pub enum FirmwareVersionCodecError {
    #[error("{0}")]
    FromUtf8Error(#[from] std::string::FromUtf8Error),
    #[error("{0}")]
    FromStrError(#[from] ParseIntError),
}

#[derive(Debug, Error)]
pub enum MakeSystemConfigError {
    #[error("firmware version codec error {0}")]
    FirmwareVersionCodecError(#[from] FirmwareVersionCodecError),
    #[error("try from slice error {0}")]
    TryFromSliceError(#[from] TryFromSliceError),
    #[error("{0}")]
    TryFromPrimitivePMError(#[from] TryFromPrimitiveError<ProductModel>),
    #[error("{0}")]
    Other(String)
}

#[derive(Debug, Error)]
pub enum ReadoutError {
    #[error("Base (legacy) protocol is not supported. Please check the Extended protocol flag in SIConfig+ or use a newer station")]
    BaseNotSupported,
    #[error("{0} is not supported.")]
    CardNotSupported(CardType)
}

#[derive(Debug, Error)]
pub enum DeserializePunchError {
    #[error("Invalid punch time")]
    InvalidTime,
    #[error("Invalid day")]
    InvalidDay,
    #[error("Invalid week")]
    InvalidWeek
}