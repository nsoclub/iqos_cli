use btleplug::api::{Peripheral as _, WriteType};

use crate::iqos::IQOS;
use crate::iqos::error::{IQOSError, Result};

#[derive(Debug, Clone, Copy)]
pub enum BrightnessLevel {
    High,
    Low,
}

pub trait IlumaFeatures: Send + Sync {
    async fn update_brightness(&self, level: BrightnessLevel) -> Result<()>;
    async fn update_autostart(&self, enable: bool) -> Result<()>;
    async fn update_smartgesture(&self, enable: bool) -> Result<()>;
}

const BRIGHTNESS_HIGH_SIGNAL_FIRST: [u8; 9] = [0x00, 0xc0, 0x46, 0x23, 0x64, 0x00, 0x00, 0x00, 0x4f];
const BRIGHTNESS_HIGH_SIGNAL_SECOND: [u8; 5] = [0x00, 0xc0, 0x02, 0x23, 0xc3];
const BRIGHTNESS_HIGH_SIGNAL_THIRD: [u8; 9] = [0x00, 0xc9, 0x44, 0x24, 0x64, 0x00, 0x00, 0x00, 0x34];
const BRIGHTNESS_LOW_SIGNAL_FIRST: [u8; 9] = [0x00, 0xc0, 0x46, 0x23, 0x1e, 0x00, 0x00, 0x00, 0xe1];
const BRIGHTNESS_LOW_SIGNAL_SECOND: [u8; 5] = [0x00, 0xc0, 0x02, 0x23, 0xc3];
const BRIGHTNESS_LOW_SIGNAL_THIRD: [u8; 9] = [0x00, 0xc9, 0x44, 0x24, 0x1e, 0x00, 0x00, 0x00, 0x9a];

const AUTOSTART_ENABLE_SIGNAL: [u8; 9] = [0x00, 0xc9, 0x47, 0x24, 0x01, 0x01, 0x00, 0x00, 0x3f];
const AUTOSTART_DISABLE_SIGNAL: [u8; 9] = [0x00, 0xc9, 0x47, 0x24, 0x01, 0x00, 0x00, 0x00, 0x54];

const SMARTGESTURE_ENABLE_SIGNAL: [u8; 9] = [0x00, 0xc9, 0x47, 0x24, 0x04, 0x01, 0x00, 0x00, 0x3c];
const SMARTGESTURE_DISABLE_SIGNAL: [u8; 9] = [0x00, 0xc9, 0x47, 0x24, 0x04, 0x00, 0x00, 0x00, 0x57];

#[derive(Debug)]
pub struct NotIlumaError;

impl std::fmt::Display for NotIlumaError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "This device is not an IQOS ILUMA model")
    }
}

impl std::error::Error for NotIlumaError {}

impl std::str::FromStr for BrightnessLevel {
    type Err = IQOSError;

    fn from_str(s: &str) -> Result<Self> {
        match s.to_lowercase().as_str() {
            "high" => Ok(BrightnessLevel::High),
            "low" => Ok(BrightnessLevel::Low),
            _ => Err(IQOSError::ConfigurationError("Invalid brightness level".to_string())),
        }
    }
}

impl std::fmt::Display for BrightnessLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BrightnessLevel::High => write!(f, "high"),
            BrightnessLevel::Low => write!(f, "low"),
        }
    }
}

impl IlumaFeatures for IQOS {
    async fn update_brightness(&self, level: BrightnessLevel) -> Result<()> {
        if !self.is_iluma() {
            return Err(IQOSError::NotIluma(NotIlumaError));
        }

        let signals: Vec<Vec<u8>> = match level {
            BrightnessLevel::High => vec!(
                BRIGHTNESS_HIGH_SIGNAL_FIRST.to_vec(),
                BRIGHTNESS_HIGH_SIGNAL_SECOND.to_vec(),
                BRIGHTNESS_HIGH_SIGNAL_THIRD.to_vec(),
            ),
            BrightnessLevel::Low => vec!(
                BRIGHTNESS_LOW_SIGNAL_FIRST.to_vec(),
                BRIGHTNESS_LOW_SIGNAL_SECOND.to_vec(),
                BRIGHTNESS_LOW_SIGNAL_THIRD.to_vec(),
            ),
        };

        for signal in signals {
            self.peripheral().write(
                self.scp_control_characteristic(),
                &signal,
                WriteType::WithResponse,
            ).await.map_err(IQOSError::BleError)?;
        }

        Ok(())
    }

    async fn update_autostart(&self, enable: bool) -> Result<()> {
        if !self.is_iluma() {
            return Err(IQOSError::NotIluma(NotIlumaError));
        }

        let signal = if enable {
            AUTOSTART_ENABLE_SIGNAL
        } else {
            AUTOSTART_DISABLE_SIGNAL
        };

        self.peripheral().write(
            self.scp_control_characteristic(),
            &signal,
            WriteType::WithResponse,
        ).await.map_err(IQOSError::BleError)?;

        Ok(())
    }

    async fn update_smartgesture(&self, enable: bool) -> Result<()> {
        if !self.is_iluma() {
            return Err(IQOSError::NotIluma(NotIlumaError));
        }

        let signal = if enable {
            SMARTGESTURE_ENABLE_SIGNAL
        } else {
            SMARTGESTURE_DISABLE_SIGNAL
        };

        self.peripheral().write(
            self.scp_control_characteristic(),
            &signal,
            WriteType::WithResponse,
        ).await.map_err(IQOSError::BleError)?;

        Ok(())
    }
}