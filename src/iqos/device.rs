use super::error::{IQOSError, Result};
use anyhow::Context;
use btleplug::api::{Characteristic, Peripheral as _, Service};
use btleplug::platform::Peripheral;
use futures::stream::StreamExt;
use uuid::{uuid, Uuid};

use std::fmt;
use std::option::Option;

const BATTERY_CHARA_UUID: Uuid = uuid!("f8a54120-b041-11e4-9be7-0002a5d5c51b");

pub struct IQOS {
    modelnumber: String,
    serialnumber: String,
    softwarerevision: String,
    manufacturername: String,
    holder_battery_status: u8,
    scp_characteristic_uuid: String,
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
            scp_characteristic_uuid: String::new(),
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

    pub async fn initialize(&mut self) -> Result<()> {
        self.peripheral
            .discover_services()
            .await
            .map_err(IQOSError::BleError)?;

        for service in self.peripheral.services() {
            for characteristic in service.characteristics {
                if characteristic.uuid.to_string().starts_with("FFE9") {
                    self.scp_characteristic_uuid = characteristic.uuid.to_string();
                    break;
                }
            }
        }

        self.update_device_info().await?;
        Ok(())
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
            .find(|chara| chara.uuid == BATTERY_CHARA_UUID)
            .ok_or(IQOSError::CharacteristicNotFound)?;

        if let Ok(data) = peripheral.read(battery_chara)
            .await
            .map_err(IQOSError::BleError) {
                let battery_status = u8::from_str_radix(&format!("{:02X}", data[2]), 16);
                self.holder_battery_status = battery_status.unwrap_or(0);
            }
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
