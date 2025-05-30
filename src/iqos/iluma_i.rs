use futures::{stream, StreamExt};

use super::device::IqosIluma;

use crate::iqos::error::{IQOSError, Result};
use crate::iqos::flexbattery::{FlexBattery, LOAD_FLEXBATTERY_SIGNAL, LOAD_PAUSEMODE_SIGNAL};
use super::iqos::IqosBle;
use btleplug::api::{Peripheral as _, WriteType};

impl IqosIlumaI for IqosBle {
    async fn update_flexbattery(&self) -> Result<()> {
        let signal = [0x00, 0xc9, 0x47, 0x24, 0x02, 0x01, 0x00, 0x00, 0x3d];
        self.send_command(signal).await
    }

    async fn load_flexbattery(&self) -> Result<Flexbattery> {
        if !self.is_iluma_i() {
            return Err(IQOSError::IncompatibleModelError);
        }

        let mut flexbattery: FlexBattery;

        self.send_command(LOAD_FLEXBATTERY_SIGNAL.to_vec()).await?;
        let mut stream = self.notifications().await?;
        if let Some(notification) = stream.next().await {
            if let Ok(mode) = FlexBattery::from_bytes(&notification.value) {
                flexbattery.mode(&mode);
            } else {
                return Err(IQOSError::ConfigurationError("Invalid flexbattery data received".to_string()))
            }
            
            let hex_string = notification.value.iter()
                .map(|byte| format!("{:02X}", byte))
                .collect::<Vec<String>>()
                .join(" ");
            println!("  Signal: {}", hex_string);
            return Ok(flexbattery);
        } else {
            return Err(IQOSError::ConfigurationError("No notifications received".to_string()));
        }
    }
}
