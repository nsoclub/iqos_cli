use std::error::Error;
use std::fmt;

use btleplug::Error as BleError;
use super::iluma::NotIlumaError;

#[derive(Debug)]
pub enum IQOSError {
    BleError(BleError),
    NotIluma(NotIlumaError),
    ConfigurationError(String),
    AutoStartError(String),
    AdapterError(String),
    IncompatibleModelError, // 互換性エラーを追加
}

impl fmt::Display for IQOSError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            IQOSError::BleError(err) => write!(f, "Bluetooth error: {}", err),
            IQOSError::NotIluma(err) => write!(f, "{}", err),
            IQOSError::ConfigurationError(msg) => write!(f, "Configuration error: {}", msg),
            IQOSError::AutoStartError(msg) => write!(f, "AutoStart error: {}", msg),
            IQOSError::AdapterError(msg) => write!(f, "Adapter error: {}", msg),
            IQOSError::IncompatibleModelError => write!(f, "Incompatible model error"),
        }
    }
}

impl Error for IQOSError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            IQOSError::BleError(err) => Some(err),
            IQOSError::NotIluma(err) => Some(err),
            IQOSError::ConfigurationError(_) => None,
            IQOSError::AutoStartError(_) => None,
            IQOSError::AdapterError(_) => None,
            IQOSError::IncompatibleModelError => None,
        }
    }
}

impl From<btleplug::Error> for IQOSError {
    fn from(error: btleplug::Error) -> Self {
        IQOSError::BleError(error)
    }
}

impl From<&str> for IQOSError {
    fn from(error: &str) -> Self {
        IQOSError::AdapterError(error.to_string())
    }
}

pub type Result<T> = std::result::Result<T, IQOSError>;