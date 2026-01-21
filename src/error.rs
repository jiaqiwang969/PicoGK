//! Error types for PicoGK

use thiserror::Error;

/// PicoGK error types
#[derive(Error, Debug)]
pub enum Error {
    /// Library initialization failed
    #[error("Failed to initialize PicoGK library")]
    InitializationFailed,

    /// Invalid parameter provided
    #[error("Invalid parameter: {0}")]
    InvalidParameter(String),

    /// Null pointer encountered
    #[error("Null pointer returned from C++ library")]
    NullPointer,

    /// IO error
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// File load error
    #[error("File load error: {0}")]
    FileLoad(String),

    /// File save error
    #[error("File save error: {0}")]
    FileSave(String),

    /// FFI error
    #[error("FFI error: {0}")]
    Ffi(String),

    /// Invalid handle
    #[error("Invalid handle")]
    InvalidHandle,

    /// Operation failed
    #[error("Operation failed: {0}")]
    OperationFailed(String),
}

/// Result type alias for PicoGK operations
pub type Result<T> = std::result::Result<T, Error>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display() {
        let err = Error::InvalidParameter("test".to_string());
        assert_eq!(err.to_string(), "Invalid parameter: test");
    }
}
