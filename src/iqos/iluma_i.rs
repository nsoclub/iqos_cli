use futures::{stream, StreamExt};

use super::device::IqosIluma;

use crate::iqos::error::{IQOSError, Result};
use super::iqos::IqosBle;
use btleplug::api::{Peripheral as _, WriteType};

impl IqosIlumaI for IqosBle {
    async fn update_flexbattery(&self) -> Result<()> {
        let signal = [0x00, 0xc9, 0x47, 0x24, 0x02, 0x01, 0x00, 0x00, 0x3d];
        self.send_command(signal).await
    }

    async fn load_flexbattery(&self) -> Result<()> {
        let signal = [0x00, 0xc9, 0x47, 0x24, 0x02, 0x00, 0x00, 0x00, 0x5a];
        self.send_command(signal).await
    }
}
