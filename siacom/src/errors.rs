use thiserror::Error;

#[derive(Debug, Error)]
pub enum SiacomError {
    #[error("io error: {0}")]
    IoError(#[from] std::io::Error),
    #[error("permission denied by user")]
    PermissionDenied,
    #[error("join error: {0}")]
    JoinError(#[from] tokio::task::JoinError),
    #[error("(usb) active config error: {0}")]
    ActiveConfigurationError(#[from] nusb::descriptors::ActiveConfigurationError),
    #[error("no bulk IN endpoint found")]
    NoBulkInEndpoint,
    #[error("no bulk OUT endpoint found")]
    NoBulkOutEndpoint,
    #[error("no suitable interface found")]
    NoSuitableInterface,
}
