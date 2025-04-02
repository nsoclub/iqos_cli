use super::device::IQOS;
use super::error::{IQOSError, Result};
use anyhow::Context;
use btleplug::platform::Peripheral;
use btleplug::api::{Service, Characteristic, Peripheral as _};
use std::collections::BTreeSet;

use uuid::{Uuid, uuid};
use btleplug::api::bleuuid::uuid_from_u16;

// デバイス情報サービスUUID
const DEVICE_INFO_SERVICE_UUID: Uuid = uuid!("0000180a-0000-1000-8000-00805f9b34fb");

// 標準特性UUID
const MODEL_NUMBER_CHAR_UUID: &str = "00002a24";
const SERIAL_NUMBER_CHAR_UUID: &str = "00002a25";
const SOFTWARE_REVISION_CHAR_UUID: &str = "00002a28";
const MANUFACTURER_NAME_CHAR_UUID: &str = "00002a29";

pub struct IQOSBuilder {
    peripheral: Option<Peripheral>,
    modelnumber: Option<String>,
    serialnumber: Option<String>,
    softwarerevision: Option<String>,
    manufacturername: Option<String>,
}

impl IQOSBuilder {
    pub fn new(peripheral: Peripheral) -> Self {
        Self {
            peripheral: Some(peripheral),
            modelnumber: None,
            serialnumber: None,
            softwarerevision: None,
            manufacturername: None,
        }
    }

    pub async fn with_peripheral(mut self, peripheral: Peripheral) -> Result<Self> {
        self.peripheral = Some(peripheral);
        
        let services = self.discover_services().await?;
        
        // イテレータを使って効率的にチェック
        if services.iter().any(|s| s.uuid == DEVICE_INFO_SERVICE_UUID) {
            self.update_device_info().await?;
        }
        
        Ok(self)
    }

    pub fn with_model_number(mut self, modelnumber: impl Into<String>) -> Self {
        self.modelnumber = Some(modelnumber.into());
        self
    }

    pub fn with_serial_number(mut self, serialnumber: impl Into<String>) -> Self {
        self.serialnumber = Some(serialnumber.into());
        self
    }

    pub fn with_software_revision(mut self, softwarerevision: impl Into<String>) -> Self {
        self.softwarerevision = Some(softwarerevision.into());
        self
    }

    pub fn with_manufacturer_name(mut self, manufacturername: impl Into<String>) -> Self {
        self.manufacturername = Some(manufacturername.into());
        self
    }

    pub async fn discover_services(&mut self) -> Result<BTreeSet<Service>> {
        let peripheral = self.peripheral
            .as_mut()
            .ok_or(IQOSError::ConfigurationError("Peripheral is required".to_string()))?;
        
        peripheral.discover_services().await
            .map_err(IQOSError::BleError)?;
        
        Ok(peripheral.services().into_iter().collect())
    }

    pub async fn connect(&mut self) -> Result<()> {
        let peripheral = self.peripheral
            .as_mut()
            .ok_or(IQOSError::ConfigurationError("Peripheral is required".to_string()))?;
        
        peripheral.connect().await
            .map_err(IQOSError::BleError)?;
        Ok(())
    }


    pub async fn is_connected(&self) -> Result<bool> {
        let peripheral = self.peripheral
            .as_ref()
            .ok_or(IQOSError::ConfigurationError("Peripheral is required".to_string()))?;
        
        peripheral.is_connected().await
            .map_err(IQOSError::BleError)
    }

    pub async fn find_service_by_uuid(&self, uuid: Uuid) -> Result<Option<Service>> {
        let peripheral = self.peripheral
            .as_ref()
            .ok_or(IQOSError::ConfigurationError("Peripheral is required".to_string()))?;
        
        // サービスのイテレータからUUIDが一致するものを検索
        Ok(peripheral.services().iter()
            .find(|s| s.uuid == uuid)
            .cloned())
    }

    pub async fn update_device_info(&mut self) -> Result<()> {
        let peripheral = self.peripheral
            .as_mut()
            .ok_or(IQOSError::ConfigurationError("Peripheral is required".to_string()))?;
        
        // デバイス情報サービスからキャラクタリスティックを読み取る
        if let Some(service) = peripheral.services().iter().find(|s| s.uuid == DEVICE_INFO_SERVICE_UUID) {
            for characteristic in &service.characteristics {
                match characteristic.uuid.to_string().split('-').next().unwrap() {
                    uuid if uuid == MODEL_NUMBER_CHAR_UUID => {
                        if let Ok(data) = peripheral.read(characteristic).await.map_err(IQOSError::BleError) {
                            if let Ok(value) = String::from_utf8(data) {
                                self.modelnumber = Some(value);
                            }
                        }
                    },
                    uuid if uuid == SERIAL_NUMBER_CHAR_UUID => {
                        if let Ok(data) = peripheral.read(characteristic).await.map_err(IQOSError::BleError) {
                            if let Ok(value) = String::from_utf8(data) {
                                self.serialnumber = Some(value);
                            }
                        }
                    },
                    uuid if uuid == SOFTWARE_REVISION_CHAR_UUID => {
                        if let Ok(data) = peripheral.read(characteristic).await.map_err(IQOSError::BleError) {
                            if let Ok(value) = String::from_utf8(data) {
                                self.softwarerevision = Some(value);
                            }
                        }
                    },
                    uuid if uuid == MANUFACTURER_NAME_CHAR_UUID => {
                        if let Ok(data) = peripheral.read(characteristic).await.map_err(IQOSError::BleError) {
                            if let Ok(value) = String::from_utf8(data) {
                                self.manufacturername = Some(value);
                            }
                        }
                    },
                    _ => {}
                }
            }
        }
        
        println!("Device Info Updated {}", self.manufacturername.clone().unwrap());
        Ok(())
    }

    pub async fn initialize(&mut self) -> Result<()> {
        let peripheral = self.peripheral
            .as_mut()
            .ok_or(IQOSError::ConfigurationError("Peripheral is required".to_string()))?;
        
        // サービスを検出
        peripheral.discover_services().await
            .map_err(IQOSError::BleError)?;
        
        // デバイス情報を取得
        if let Some(properties) = peripheral.properties().await
            .map_err(IQOSError::BleError)? {
            
            if let Some(name) = properties.local_name {
                self.modelnumber = Some(name.to_string());
            }
            
            // 製造者情報の処理
            let manufacturer_str = properties.manufacturer_data
                .iter()
                .map(|(id, data)| format!("ID: {}, Data: {:?}", id, data))
                .collect::<Vec<_>>()
                .join(", ");
            self.manufacturername = Some(manufacturer_str);
        }

        Ok(())
    }

    pub fn build(self) -> Result<IQOS> {
        let peripheral = self.peripheral
            .ok_or(IQOSError::ConfigurationError("Peripheral is required".to_string()))?;

        let iqos = IQOS::new(
            peripheral,
            self.modelnumber.unwrap_or_else(|| "Unknown".to_string()),
            self.serialnumber.ok_or(IQOSError::ConfigurationError("Serial number is required".to_string()))?,
            self.softwarerevision.unwrap_or_else(|| "Unknown".to_string()),
            self.manufacturername.unwrap_or_else(|| "Unknown".to_string()),
        );

        Ok(iqos)
    }

    // 生のPeripheralを取得するメソッドを追加
    pub fn peripheral(&self) -> Result<&Peripheral> {
        self.peripheral
            .as_ref()
            .ok_or(IQOSError::ConfigurationError("Peripheral is required".to_string()))
    }
}

// impl Default for IQOSBuilder {
//     fn default() -> Self {
//         Self::new(Peripheral::default())
//     }
// } 