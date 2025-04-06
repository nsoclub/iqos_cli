use super::error::{IQOSError, Result};
use super::{
    BATTERY_CHARACTERISTIC_UUID,
    SCP_CONTROL_CHARACTERISTIC_UUID,
};
use btleplug::api::{Characteristic, Peripheral as _, WriteType};
use btleplug::platform::Peripheral;
use std::fmt;
use std::collections::BTreeSet;

const START_VIBRATION_SIGNAL: [u8; 9] = [0x00, 0xc0, 0x45, 0x22, 0x01, 0x1e, 0x00, 0x00, 0xc3];

pub struct IQOS {
    modelnumber: String,
    serialnumber: String,
    softwarerevision: String,
    manufacturername: String,
    holder_battery_status: u8,
    peripheral: Peripheral,
}

impl IQOS {
    pub(crate) fn new(
        peripheral: Peripheral,
        modelnumber: String,
        serialnumber: String,
        softwarerevision: String,
        manufacturername: String,
    ) -> Self {
        Self {
            peripheral,
            modelnumber,
            serialnumber,
            softwarerevision,
            manufacturername,
            holder_battery_status: 0,
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

    async fn update_device_info(&mut self) -> Result<()> {
        // ここでデバイスの情報を取得して各フィールドを更新
        // モデル番号、シリアル番号、ソフトウェアバージョン、製造者名など
        // 実際の実装はデバイスのプロトコルに依存します

        Ok(())
    }

    pub async fn reload_battery(& mut self) -> Result<()> {
        let peripheral = &self.peripheral;

        // サービスとキャラクタリスティックを取得
        let characteristics = peripheral.characteristics();
        let battery_chara = characteristics
            .iter()
            .find(|chara| chara.uuid == BATTERY_CHARACTERISTIC_UUID)
            .ok_or(IQOSError::CharacteristicNotFound)?;

        if let Ok(data) = peripheral.read(battery_chara)
            .await
            .map_err(IQOSError::BleError) {
                let battery_status = u8::from_str_radix(&format!("{:02X}", data[2]), 16);
                self.holder_battery_status = battery_status.unwrap_or(0);
            }
        Ok(())
    }

    pub async fn vibrate(&self) -> Result<()> {
        let peripheral = &self.peripheral;

        // SCPキャラクタリスティックを取得
        let characteristics = peripheral.characteristics();
        let scp_characteristic = characteristics
            .iter()
            .find(|chara| chara.uuid.to_string() == SCP_CONTROL_CHARACTERISTIC_UUID.to_string())
            .ok_or(IQOSError::CharacteristicNotFound)?;

        // SCPキャラクタリスティックに書き込み
        peripheral.write(
            scp_characteristic,
            &START_VIBRATION_SIGNAL,
            WriteType::WithResponse,
        ).await.map_err(IQOSError::BleError)?;

        Ok(())
    }

    pub fn peripheral(&self) -> &Peripheral {
        &self.peripheral
    }

    pub fn characteristics(&self) -> BTreeSet<Characteristic> {
        self.peripheral.characteristics()
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
