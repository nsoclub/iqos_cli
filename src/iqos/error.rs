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
}

impl fmt::Display for IQOSError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            IQOSError::BleError(err) => write!(f, "Bluetooth error: {}", err),
            IQOSError::NotIluma(err) => write!(f, "{}", err),
            IQOSError::ConfigurationError(msg) => write!(f, "Configuration error: {}", msg),
            IQOSError::AutoStartError(msg) => write!(f, "AutoStart error: {}", msg),
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
        }
    }
}

pub type Result<T> = std::result::Result<T, IQOSError>;