use super::error::{IQOSError, Result};
use super::{
    BATTERY_CHARACTERISTIC_UUID,
    SCP_CONTROL_CHARACTERISTIC_UUID,
};
use btleplug::api::{Characteristic, Peripheral as _, WriteType};
use btleplug::platform::Peripheral;
use std::fmt;
use std::collections::BTreeSet;

const START_VIBRATE_SIGNAL: [u8; 9] = [0x00, 0xc0, 0x45, 0x22, 0x01, 0x1e, 0x00, 0x00, 0xc3];
const STPOP_VIBRATE_SIGNAL: [u8; 9] = [0x00, 0xc0, 0x45, 0x22, 0x00, 0x1e, 0x00, 0x00, 0xd5];

pub struct IQOS {
    modelnumber: String,
    serialnumber: String,
    softwarerevision: String,
    manufacturername: String,
    holder_battery_status: u8,
    peripheral: Peripheral,
    battery_characteristic: Characteristic,
    scp_control_characteristic: Characteristic,
}

impl IQOS {
    pub(crate) fn new(
        peripheral: Peripheral,
        modelnumber: String,
        serialnumber: String,
        softwarerevision: String,
        manufacturername: String,
        battery_characteristic: Characteristic,
        scp_control_characteristic: Characteristic,
    ) -> Self {
        Self {
            peripheral,
            modelnumber,
            serialnumber,
            softwarerevision,
            manufacturername,
            holder_battery_status: 0,
            battery_characteristic,
            scp_control_characteristic,
        }
    }

    pub fn model_number(&self) -> &str {
        &self.modelnumber
    }

    pub fn serial_number(&self) -> &str {
        &self.serialnumber
    }

    pub fn software_revision(&self) -> &str {
        &self.softwarerevision
    }

    pub fn manufacturer_name(&self) -> &str {
        &self.manufacturername
    }

    pub fn battery_status(&self) -> u8 {
        self.holder_battery_status
    }

    pub async fn disconnect(&mut self) -> Result<()> {
        let peripheral = self.peripheral.clone();

        peripheral.disconnect().await.map_err(IQOSError::BleError)
    }

    pub async fn reload_battery(& mut self) -> Result<()> {
        let peripheral = &self.peripheral;

        if let Ok(data) = peripheral.read(&self.battery_characteristic)
            .await
            .map_err(IQOSError::BleError) {
                let battery_status = u8::from_str_radix(&format!("{:02X}", data[2]), 16);
                self.holder_battery_status = battery_status.unwrap_or(0);
            }
        Ok(())
    }

    pub async fn vibrate(&self) -> Result<()> {
        let peripheral = &self.peripheral;

        peripheral.write(
            &self.scp_control_characteristic,
            &START_VIBRATE_SIGNAL,
            WriteType::WithResponse,
        ).await.map_err(IQOSError::BleError)?;

        Ok(())
    }

    pub async fn stop_vibrate(&self) -> Result<()> {
        let peripheral = &self.peripheral;

        peripheral.write(
            &self.scp_control_characteristic,
            &STPOP_VIBRATE_SIGNAL,
            WriteType::WithResponse,
        ).await.map_err(IQOSError::BleError)?;

        Ok(())
    }
}

impl fmt::Display for IQOS {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Model Number: {}\nSerial Number: {}\nSoftware Revision: {}\nManufacture Name: {}",
            self.modelnumber, self.serialnumber, self.softwarerevision, self.manufacturername,
        )
    }
}
