use futures::StreamExt;

use crate::iqos::error::{IQOSError, Result};
use crate::iqos::vibration::{VibrationSettings, LOAD_VIBRATION_SETTINGS_SIGNAL};

use super::device::IqosIluma;
use super::iqos::IqosBle;
use super::vibration::IlumaVibrationBehavior;
use btleplug::api::{Peripheral as _, WriteType};

pub struct IlumaSpecific {
    holder_product_number: String,
    firmware_version: String,
}

impl IlumaSpecific {
    pub fn new(holder_product_number: String, firmware_version: String) -> Self {
        Self {
            holder_product_number,
            firmware_version,
        }
    }

    pub fn holder_product_number(&self) -> &str {
        &self.holder_product_number
    }
}

pub const FLEXPUFF_ENABLE_SIGNAL: [u8; 9] = [0x00, 0xd2, 0x45, 0x22, 0x03, 0x01, 0x00, 0x00, 0x0A];
pub const FLEXPUFF_DISABLE_SIGNAL: [u8; 9] = [0x00, 0xd2, 0x45, 0x22, 0x03, 0x00, 0x00, 0x00, 0x0A];
pub const AUTOSTART_ENABLE_SIGNAL: [u8; 9] = [0x00, 0xc9, 0x47, 0x24, 0x01, 0x01, 0x00, 0x00, 0x3f];
pub const AUTOSTART_DISABLE_SIGNAL: [u8; 9] = [0x00, 0xc9, 0x47, 0x24, 0x01, 0x00, 0x00, 0x00, 0x54];

pub const SMARTGESTURE_ENABLE_SIGNAL: [u8; 9] = [0x00, 0xc9, 0x47, 0x24, 0x04, 0x01, 0x00, 0x00, 0x3c];
pub const SMARTGESTURE_DISABLE_SIGNAL: [u8; 9] = [0x00, 0xc9, 0x47, 0x24, 0x04, 0x00, 0x00, 0x00, 0x57];

#[derive(Debug)]
pub struct NotIlumaError;

impl std::fmt::Display for NotIlumaError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "This device is not an IQOS ILUMA model")
    }
}

impl std::error::Error for NotIlumaError {}

impl IqosIluma for IqosBle {
    async fn load_iluma_vibration_settings(&self) -> Result<VibrationSettings> {
        if !self.is_iluma() {
            return Err(IQOSError::IncompatibleModelError);
        }

        self.send_command(LOAD_VIBRATION_SETTINGS_SIGNAL.to_vec()).await?;
        let mut stream = self.notifications().await?;

        if let Some(notification) = stream.next().await {
            if let Ok(settings) = VibrationSettings::from_bytes(&notification.value) {
                return Ok(settings);
            } else {
                return Err(IQOSError::ConfigurationError("Failed to parse vibration settings".to_string()));
            }
        } else {
            return Err(IQOSError::ConfigurationError("No notifications received".to_string()));
        }
    }

    async fn update_iluma_vibration_settings(&self, settings: VibrationSettings) -> Result<()> {
        if !self.is_iluma() {
            return Err(IQOSError::IncompatibleModelError);
        }

        if let Some(iluma) = settings.as_iluma() {
            let signals = iluma.build();
            for (i, signal) in signals.iter().enumerate() {
                let hex_string = signal.iter()
                    .map(|byte| format!("{:02X}", byte))
                    .collect::<Vec<String>>()
                    .join(" ");
                println!("  Signal {}: {}", i, hex_string);
            }
            
            for signal in signals {
                self.send_command(signal).await?;
            }
        }

        Ok(())
    }

    async fn update_smartgesture(&self, enable: bool) -> Result<()> {
        if !self.is_iluma() {
            return Err(IQOSError::IncompatibleModelError);
        }

        let signal = if enable {
            SMARTGESTURE_ENABLE_SIGNAL
        } else {
            SMARTGESTURE_DISABLE_SIGNAL
        };

        let peripheral = self.peripheral();
        let characteristic = self.scp_control_characteristic();
        
        peripheral.write(
            characteristic,
            &signal,
            WriteType::WithResponse,
        ).await.map_err(IQOSError::BleError)?;

        Ok(())
    }

    async fn update_autostart(&self, enable: bool) -> Result<()> {
        if !self.is_iluma() {
            return Err(IQOSError::IncompatibleModelError);
        }

        let signal = if enable {
            AUTOSTART_ENABLE_SIGNAL
        } else {
            AUTOSTART_DISABLE_SIGNAL
        };

        let peripheral = self.peripheral();
        let characteristic = self.scp_control_characteristic();
        
        peripheral.write(
            characteristic,
            &signal,
            WriteType::WithResponse,
        ).await.map_err(IQOSError::BleError)?;

        Ok(())
    }

    async fn update_flexpuff(&self, enable: bool) -> Result<()> {
        if !self.is_iluma() {
            return Err(IQOSError::IncompatibleModelError);
        }

        let signal = if enable {
            FLEXPUFF_ENABLE_SIGNAL
        } else {
            FLEXPUFF_DISABLE_SIGNAL
        };

        let peripheral = self.peripheral();
        let characteristic = self.scp_control_characteristic();
        
        peripheral.write(
            characteristic,
            &signal,
            WriteType::WithResponse,
        ).await.map_err(IQOSError::BleError)?;

        Ok(())
    }
}