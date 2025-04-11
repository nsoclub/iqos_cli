use super::error::{IQOSError, Result};
use super::device::Iqos;
use super::COMMAND_CHECKSUM_XOR;
use btleplug::api::{Characteristic, Peripheral as _, WriteType};
use btleplug::platform::Peripheral;

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

impl std::fmt::Display for IQOSModel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            IQOSModel::One => write!(f, "ONE"),
            IQOSModel::Iluma => write!(f, "ILUMA"),
        }
    }
}

impl IQOSModel {
    pub async fn from_peripheral(peripheral: &Peripheral) -> Self {
        if let Ok(properties) = peripheral.properties().await {
            if let Some(properties) = properties {
                if let Some(name) = properties.local_name {
                    if name.contains("ONE") {
                        return IQOSModel::One;
                    }
                }
            }
        }
        IQOSModel::Iluma // デフォルトはIlumaとする
    }
}


pub struct IqosBle {
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

impl IqosBle {
    pub(crate) async fn new(
        peripheral: Peripheral,
        modelnumber: String,
        serialnumber: String,
        softwarerevision: String,
        manufacturername: String,
        battery_characteristic: Characteristic,
        scp_control_characteristic: Characteristic,
    ) -> Self {
        let model = IQOSModel::from_peripheral(&peripheral).await;
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

    async fn send_command(&self, command: Vec<u8>) -> Result<()> {
        let peripheral = &self.peripheral;
        
        peripheral.write(
            &self.scp_control_characteristic,
            &self.with_checksum(command),
            WriteType::WithResponse,
        ).await.map_err(IQOSError::BleError)?;
        
        Ok(())
    }
    

    pub async fn send_command_sequence(&self, commands: Vec<Vec<u8>>) -> Result<()> {
        for command in commands {
            self.send_command(command).await?;
        }
        Ok(())
    }

    pub fn as_iluma(&self) -> Option<&IqosBle> {
        match self.model {
            IQOSModel::Iluma => Some(self),
            _ => None,
        }
    }

    pub fn is_iluma(&self) -> bool {
        matches!(self.model, IQOSModel::Iluma)
    }

    pub fn model(&self) -> &IQOSModel {
        &self.model
    }

    // フィールド名と同じ名前のgetterメソッド
    pub(crate) fn peripheral(&self) -> &Peripheral {
        &self.peripheral
    }

    pub(crate) fn scp_control_characteristic(&self) -> &Characteristic {
        &self.scp_control_characteristic
    }

    pub(crate) fn battery_characteristic(&self) -> &Characteristic {
        &self.battery_characteristic
    }

    fn calculate_checksum(&self, command: &Vec<u8>) -> u8 {
        let sum: u8 = command.iter().fold(0u8, |acc, &x| acc.wrapping_add(x));
        
        sum ^ COMMAND_CHECKSUM_XOR
    }

    pub fn with_checksum(&self, command: Vec<u8>) -> Vec<u8> {
        let mut cmd = command;
        let checksum = self.calculate_checksum(&cmd);
        cmd.push(checksum);
        cmd
    }

}

impl Iqos for IqosBle {
    async fn disconnect(&mut self) -> Result<()> {
        self.peripheral.disconnect().await.map_err(IQOSError::BleError)
    }
    
    async fn reload_battery(&mut self) -> Result<()> {
        let peripheral = &self.peripheral;

        if let Ok(data) = peripheral.read(&self.battery_characteristic)
            .await
            .map_err(IQOSError::BleError) {
                let battery_status = u8::from_str_radix(&format!("{:02X}", data[2]), 16);
                self.holder_battery_status = battery_status.unwrap_or(0);
            }
        Ok(())
    }
    
    fn battery_status(&self) -> u8 {
        self.holder_battery_status
    }
    
    async fn vibrate(&self) -> Result<()> {
        let peripheral = &self.peripheral;

        peripheral.write(
            &self.scp_control_characteristic,
            &START_VIBRATE_SIGNAL,
            WriteType::WithResponse,
        ).await.map_err(IQOSError::BleError)?;

        Ok(())
    }
    
    async fn stop_vibrate(&self) -> Result<()> {
        let peripheral = &self.peripheral;

        peripheral.write(
            &self.scp_control_characteristic,
            &STOP_VIBRATE_SIGNAL,
            WriteType::WithResponse,
        ).await.map_err(IQOSError::BleError)?;

        Ok(())
    }
    
    async fn lock_device(&self) -> Result<()> {
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
    
    async fn unlock_device(&self) -> Result<()> {
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
}

impl std::fmt::Display for IqosBle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Model: {}\nModel Number: {}\nSerial Number: {}\nSoftware Revision: {}\nManufacturer Name: {}\n",
            self.model,
            self.modelnumber,
            self.serialnumber,
            self.softwarerevision,
            self.manufacturername,
        )
    }
}