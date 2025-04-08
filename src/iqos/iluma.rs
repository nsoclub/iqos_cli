use btleplug::api::{Peripheral as _, WriteType};
use futures::stream::ForEach;

use crate::iqos::IQOS;
use crate::iqos::error::{IQOSError, Result};

#[derive(Debug, Clone, Copy)]
pub enum BrightnessLevel {
    High,
    Low,
}

pub trait IlumaFeatures: Send + Sync {
    async fn update_brightness(&self, level: BrightnessLevel) -> Result<()>;
}

const BRIGHTNESS_HIGH_SIGNAL_FIRST: [u8; 9] = [0x00, 0xc0, 0x46, 0x23, 0x64, 0x00, 0x00, 0x00, 0x4f];
const BRIGHTNESS_LOW_SIGNAL_FIRST: [u8; 9] = [0x00, 0xc0, 0x46, 0x23, 0x1e, 0x00, 0x00, 0x00, 0xe1];

#[derive(Debug)]
pub struct NotIlumaError;

impl std::fmt::Display for NotIlumaError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "This device is not an IQOS ILUMA model")
    }
}

impl std::error::Error for NotIlumaError {}

impl IlumaFeatures for IQOS {
    async fn update_brightness(&self, level: BrightnessLevel) -> Result<()> {
        if !self.is_iluma() {
            return Err(IQOSError::NotIluma(NotIlumaError));
        }

        let signals: Vec<Vec<u8>> = match level {
            BrightnessLevel::High => vec!(BRIGHTNESS_HIGH_SIGNAL_FIRST.to_vec(),),
            BrightnessLevel::Low => vec!(BRIGHTNESS_LOW_SIGNAL_FIRST.to_vec(),),
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
}