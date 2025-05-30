use futures::{stream, StreamExt};

use crate::iqos::error::{IQOSError, Result};
use crate::iqos::flexbattery::{FlexBattery, LOAD_FLEXBATTERY_SIGNAL, LOAD_PAUSEMODE_SIGNAL};
use super::iqos::IqosBle;
use super::device::IqosIlumaI;

impl IqosIlumaI for IqosBle {
    async fn update_flexbattery(&self, new: FlexBattery) -> Result<()> {
        self.send_command(new.mode().to_bytes()).await?;
        let hex_string = new.mode().to_bytes().iter()
            .map(|byte| format!("{:02X}", byte))
            .collect::<Vec<String>>()
            .join(" ");
        println!("  Signal: {}", hex_string);
        if new.is_performance() {
            if let Some(pausemode) = new.is_pausemode() {
                self.send_command(FlexBattery::pausemode_to_bytes(pausemode)).await?;
            }
        }
        Ok(())
    }

    async fn load_flexbattery(&self) -> Result<FlexBattery> {
        if !self.is_iluma_i() {
            return Err(IQOSError::IncompatibleModelError);
        }

        let mut flexbattery: FlexBattery = Default::default();

        self.send_command(LOAD_FLEXBATTERY_SIGNAL.to_vec()).await?;
        let mut stream = self.notifications().await?;
        if let Some(notification) = stream.next().await {
            if let Ok(mode) = FlexBattery::from_bytes(&notification.value) {
                flexbattery.update_mode(&mode);
            } else {
                return Err(IQOSError::ConfigurationError("Invalid flexbattery data received".to_string()))
            }
            
            let hex_string = notification.value.iter()
                .map(|byte| format!("{:02X}", byte))
                .collect::<Vec<String>>()
                .join(" ");
            println!("  Signal: {}", hex_string);
        } else {
            return Err(IQOSError::ConfigurationError("No notifications received".to_string()));
        }
        if flexbattery.is_performance() {
            self.send_command(LOAD_PAUSEMODE_SIGNAL.to_vec()).await?;
            let mut stream = self.notifications().await?;
            if let Some(notification) = stream.next().await {
                if let Ok(pause_mode) = FlexBattery::pausemode_from_bytes(&notification.value) {
                    flexbattery.update_pause_mode(pause_mode);
                } else {
                    return Err(IQOSError::ConfigurationError("Invalid pause mode data received".to_string()));
                }
                
                let hex_string = notification.value.iter()
                    .map(|byte| format!("{:02X}", byte))
                    .collect::<Vec<String>>()
                    .join(" ");
                println!("  Signal: {}", hex_string);
            } else {
                return Err(IQOSError::ConfigurationError("No notifications received for pause mode".to_string()));
            }
        }
        Ok(flexbattery)
    }
}
