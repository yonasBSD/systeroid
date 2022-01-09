use thiserror::Error as ThisError;

/// Custom error type.
#[derive(Debug, ThisError)]
pub enum Error {
    /// Error that may occur during I/O operations.
    #[error("IO error: `{0}`")]
    IoError(#[from] std::io::Error),
    /// Error that may occur while receiving messages from the channel.
    #[error("Channel receive error: `{0}`")]
    ReceiveError(#[from] std::sync::mpsc::RecvError),
    /// Error that may occur in the core library.
    #[error("{0}")]
    SysctlError(#[from] systeroid_core::error::Error),
}

/// Type alias for the standard [`Result`] type.
pub type Result<T> = core::result::Result<T, Error>;