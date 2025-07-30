use std::{array::TryFromSliceError, num::ParseIntError};

use num_enum::TryFromPrimitiveError;
use thiserror::Error;

use crate::{card::CardType, product::ProductModel};

#[derive(Debug, Error)]
pub enum NewConnectionError {
    #[cfg(not(target_os = "android"))]
    #[error("SerialPort error: {0}")]
    SerialportError(#[from] tokio_serial::Error),
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
    #[error("ReceiveRawPacket error: {0}")]
    ReceiveRawPacketError(#[from] ReceiveRawPacketError),
    #[error("ReceivePacket error: {0}")]
    ReceivePacketError(#[from] ReceivePacketError),
    #[error("connop error: {0}")]
    ConnectionOperationError(#[from] ConnectionOperationError),
    #[error("MakeSystemConfig error: {0}")]
    MakeSystemConfigError(#[from] MakeSystemConfigError),
    #[error("Try from slice error {0}")]
    TryFromSliceError(#[from] TryFromSliceError),
    #[error("Failed to set msmode to MASTER")]
    FailedToSetMsMode,
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
pub enum ReceivePacketError {
    #[error("receive raw packet error {0}")]
    ReceiveRawPacketError(#[from] ReceiveRawPacketError),
    #[error("deserialize packet error: {0}")]
    DeserializePacketError(#[from] DeserializePacketError),
}

#[derive(Debug, Error)]
pub enum ReceiveRawPacketError {
    #[error("deserialize raw packet error {0}")]
    DeserializeRawPacketError(#[from] DeserializeRawPacketError),
    //#[error("Timed out")]
    //TimedOut
}

#[derive(Debug, Error)]
pub enum DeserializeRawPacketError {
    #[cfg(not(target_os = "android"))]
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
    Other(String),
}

#[derive(Debug, Error)]
pub enum ReadoutError {
    #[error(
        "Base (legacy) protocol is not supported. Please check the Extended protocol flag in SIConfig+ or use a newer station"
    )]
    BaseNotSupported,
    #[error("{0} is not supported.")]
    CardNotSupported(CardType),
    #[error("Feed block error: {0}")]
    FeedBlockError(#[from] FeedBlockError),
    #[error("Expected {0} but got {1}")]
    ExpectedButGot(CardType, CardType),
    #[error("Deserialize packet error: {0}")]
    ReceivePacketError(#[from] ReceivePacketError),
    #[error("Could not get the card type")]
    CouldNotGetCardType,
    #[error("io error {0}")]
    IoError(#[from] std::io::Error),
}

#[derive(Debug, Error)]
pub enum DeserializePunchError {
    #[error("Invalid punch time")]
    InvalidTime,
    #[error("Invalid day")]
    InvalidDay,
    #[error("Invalid week")]
    InvalidWeek,
    #[error("The input is cleared (0xFF)")]
    DataCleared,
    #[error("Invalid data")]
    InvalidData,
}

#[derive(Debug, Error)]
pub enum DeserializeCardPersonalDataError {
    #[error("Required fields (first and last name) are empty")]
    RequiredFieldsAreEmpty,
    #[error("From UTF8 error: {0}")]
    FromUtf8Error(#[from] std::string::FromUtf8Error),
}

#[derive(Debug, Error)]
pub enum FeedBlockError {
    #[error("The provided block buffer does not match with the block id")]
    BlockIdAndBufferNotMatching,
    #[error("This card definition does no have that block")]
    BlockDoesNotExist,
    #[error("Deserialize block error: {0}")]
    DeserializeBlockError(#[from] DeserializeBlockError),
}

#[derive(Debug, Error)]
pub enum DeserializeBlockError {
    #[error("Deserialize punch error {0}")]
    DeserializePunchError(#[from] DeserializePunchError),
    #[error("Could not deserialize a date")]
    DateError,
}

#[derive(Debug, Error)]
pub enum ConnectionOperationError {
    #[error("io error {0}")]
    IoError(#[from] std::io::Error),
    #[error("receive packet error: {0}")]
    ReceivePacketError(#[from] ReceivePacketError),
    #[error("receive raw packet error {0}")]
    ReceiveRawPacketError(#[from] ReceiveRawPacketError),
}

#[derive(Debug, Error)]
pub enum ReadoutResultTransformationError {
    #[error("SIID is None")]
    SiidNone,
    #[error("Clear/Check punch is None")]
    ClearCheckNone,
    #[error("Deserialize card personal data error: {0}")]
    DeserializeCardPersonalDataError(#[from] DeserializeCardPersonalDataError),
    #[error("Punches are None")]
    PunchesNone,
}

#[derive(Debug, Error)]
pub enum SimpleActionError {
    #[error("io error {0}")]
    IoError(#[from] std::io::Error),
    #[error("receive packet error: {0}")]
    ReceivePacketError(#[from] ReceivePacketError),
    #[error("receive raw packet error {0}")]
    ReceiveRawPacketError(#[from] ReceiveRawPacketError),
}
