use thiserror::Error;

#[derive(Error, Debug)]
pub enum IQOSError {
    #[error("デバイスが初期化されていません")]
    DeviceNotInitialized,
    #[error("BLEエラー: {0}")]
    BleError(#[from] btleplug::Error),
    #[error("デバイス設定エラー: {0}")]
    ConfigurationError(String),
}

pub type Result<T> = std::result::Result<T, IQOSError>; 