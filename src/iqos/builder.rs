use super::device::IQOS;
use super::error::{IQOSError, Result};
use super::{
    BATTERY_CHARACTERISTIC_UUID, CORE_SERVICE_UUID, DEVICE_INFO_SERVICE_UUID, MANUFACTURER_NAME_CHAR_UUID, MODEL_NUMBER_CHAR_UUID, SERIAL_NUMBER_CHAR_UUID, SOFTWARE_REVISION_CHAR_UUID, SCP_CONTROL_CHARACTERISTIC_UUID
};
use btleplug::platform::Peripheral;
use btleplug::api::{Characteristic, Peripheral as _, Service};
use std::collections::BTreeSet;
use uuid::Uuid;

pub struct IQOSBuilder {
    peripheral: Option<Peripheral>,
    modelnumber: Option<String>,
    serialnumber: Option<String>,
    softwarerevision: Option<String>,
    manufacturername: Option<String>,
    battery_characteristic: Option<Characteristic>,
    scp_control_characteristic: Option<Characteristic>,
}

impl IQOSBuilder {
    pub fn new(peripheral: Peripheral) -> Self {
        Self {
            peripheral: Some(peripheral),
            modelnumber: None,
            serialnumber: None,
            softwarerevision: None,
            manufacturername: None,
            battery_characteristic: None,
            scp_control_characteristic: None,
        }
    }

    pub async fn with_peripheral(mut self, peripheral: Peripheral) -> Result<Self> {
        self.peripheral = Some(peripheral);
        
        let services = self.discover_services().await?;
        
        // イテレータを使って効率的にチェック
        if services.iter().any(|s| s.uuid == DEVICE_INFO_SERVICE_UUID) {
            self.load_device_info().await?;
            self.load_characteristics().await?;
        }
        
        Ok(self)
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

    async fn load_device_info(&mut self) -> Result<()> {
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
        
        Ok(())
    }

    async fn load_characteristics(&mut self) -> Result<()> {
        let peripheral = self.peripheral
            .as_mut()
            .ok_or(IQOSError::ConfigurationError("Peripheral is required".to_string()))?;
        
        if let Some(service) = peripheral.services().iter().find(|s| s.uuid == CORE_SERVICE_UUID) {
            for characteristic in &service.characteristics {
                let uuid = characteristic.uuid;
                if uuid == BATTERY_CHARACTERISTIC_UUID {
                    self.battery_characteristic = Some(characteristic.clone());
                } else if uuid == SCP_CONTROL_CHARACTERISTIC_UUID {
                    self.scp_control_characteristic = Some(characteristic.clone());
                }
            }
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
            self.battery_characteristic.ok_or(IQOSError::ConfigurationError("Battery characteristic is required".to_string()))?,
            self.scp_control_characteristic.ok_or(IQOSError::ConfigurationError("SCP Control characteristic is required".to_string()))?,
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