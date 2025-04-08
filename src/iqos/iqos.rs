use super::error::{IQOSError, Result};
use btleplug::api::{Characteristic, Peripheral as _, WriteType};
use btleplug::platform::Peripheral;
use std::fmt;

const CONFIRMATION_SIGNAL: [u8; 5] = [0x00, 0xc0, 0x01, 0x00, 0x15];
const START_VIBRATE_SIGNAL: [u8; 9] = [0x00, 0xc0, 0x45, 0x22, 0x01, 0x1e, 0x00, 0x00, 0xc3];
const STOP_VIBRATE_SIGNAL: [u8; 9] = [0x00, 0xc0, 0x45, 0x22, 0x00, 0x1e, 0x00, 0x00, 0xd5];
const LOCK_SIGNAL_FIRST: [u8; 9] = [0x00, 0xc9, 0x44, 0x04, 0x02, 0xff, 0x00, 0x00, 0x5a];
const LOCK_SIGNAL_SECOND: [u8; 5] = [0x00, 0xc9, 0x00, 0x04, 0x1c];
const UNLOCK_SIGNAL_FIRST: [u8; 9] = [0x00, 0xc9, 0x44, 0x04, 0x00, 0x00, 0x00, 0x00, 0x5d];
const UNLOCK_SIGNAL_SECOND: [u8; 5] = [0x00, 0xc9, 0x00, 0x04, 0x1c];

#[derive(Debug, Clone, PartialEq)]
pub enum IQOSModel {
    One,
    Iluma,
}

impl IQOSModel {
    pub async fn from_peripheral(peripheral: Peripheral) -> Self {
        if let Some(name) = peripheral.properties().await.unwrap().unwrap().local_name {
            if name.contains("ONE") {
                IQOSModel::One
            } else {
                IQOSModel::Iluma
            }
        } else {
            IQOSModel::One
        }
    }
}

pub struct IQOS {
    modelnumber: String,
    serialnumber: String,
    softwarerevision: String,
    manufacturername: String,
    holder_battery_status: u8,
    peripheral: Peripheral,
    battery_characteristic: Characteristic,
    scp_control_characteristic: Characteristic,
    model: IQOSModel,
}

impl IQOS {
    pub(crate) async fn new(
        peripheral: Peripheral,
        modelnumber: String,
        serialnumber: String,
        softwarerevision: String,
        manufacturername: String,
        battery_characteristic: Characteristic,
        scp_control_characteristic: Characteristic,
    ) -> Self {
        let model = IQOSModel::from_peripheral(peripheral.clone()).await;
        Self {
            peripheral,
            modelnumber,
            serialnumber,
            softwarerevision,
            manufacturername,
            holder_battery_status: 0,
            battery_characteristic,
            scp_control_characteristic,
            model,
        }
    }

    pub fn battery_status(&self) -> u8 {
        self.holder_battery_status
    }

    pub async fn disconnect(&mut self) -> Result<()> {
        let peripheral = self.peripheral.clone();

        peripheral.disconnect().await.map_err(IQOSError::BleError)
    }

    pub fn as_iluma(&self) -> Option<&IQOS> {
        match self.model {
            IQOSModel::Iluma => Some(self),
            _ => None,
        }
    }

    pub fn is_iluma(&self) -> bool {
        matches!(self.model, IQOSModel::Iluma)
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
            &STOP_VIBRATE_SIGNAL,
            WriteType::WithResponse,
        ).await.map_err(IQOSError::BleError)?;

        Ok(())
    }

    pub async fn lock_device(&self) -> Result<()> {
        let peripheral = &self.peripheral;

        peripheral.write(
            &self.scp_control_characteristic,
            &LOCK_SIGNAL_FIRST,
            WriteType::WithResponse,
        ).await.map_err(IQOSError::BleError)?;

        peripheral.write(
            &self.scp_control_characteristic,
            &LOCK_SIGNAL_SECOND,
            WriteType::WithResponse,
        ).await.map_err(IQOSError::BleError)?;

        peripheral.write(
            &self.scp_control_characteristic,
            &CONFIRMATION_SIGNAL,
            WriteType::WithResponse,
        ).await.map_err(IQOSError::BleError)?;

        Ok(())
    }

    pub async fn unlock_device(&self) -> Result<()> {
        let peripheral = &self.peripheral;

        peripheral.write(
            &self.scp_control_characteristic,
            &UNLOCK_SIGNAL_FIRST,
            WriteType::WithResponse,
        ).await.map_err(IQOSError::BleError)?;

        peripheral.write(
            &self.scp_control_characteristic,
            &UNLOCK_SIGNAL_SECOND,
            WriteType::WithResponse,
        ).await.map_err(IQOSError::BleError)?;

        peripheral.write(
            &self.scp_control_characteristic,
            &CONFIRMATION_SIGNAL,
            WriteType::WithResponse,
        ).await.map_err(IQOSError::BleError)?;

        Ok(())
    }

    pub(crate) fn peripheral(&self) -> &Peripheral {
        &self.peripheral
    }

    pub(crate) fn scp_control_characteristic(&self) -> &Characteristic {
        &self.scp_control_characteristic
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
