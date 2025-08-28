use thiserror::Error;

#[derive(Debug, Error)]
pub enum SiacomError {
    #[error("io error: {0}")]
    IoError(#[from] std::io::Error),
    #[error("nusb error: {0}")]
    NusbError(#[from] nusb::Error),
    #[error("nusb - active configuration error: {0}")]
    ActiveConfigrationError(#[from] nusb::ActiveConfigurationError),
    #[error("permission denied by user")]
    PermissionDenied,
    #[error("join error: {0}")]
    JoinError(#[from] tokio::task::JoinError),
    #[error("no bulk IN endpoint found")]
    NoBulkInEndpoint,
    #[error("no bulk OUT endpoint found")]
    NoBulkOutEndpoint,
    #[error("no suitable interface found")]
    NoSuitableInterface,
}
