use super::error::Result;
use super::iluma::{BrightnessLevel, VibrationSettings};

/// すべてのIQOSデバイスに共通する基本機能
pub trait Iqos {
    async fn disconnect(&mut self) -> Result<()>;
    
    async fn reload_battery(&mut self) -> Result<()>;
    
    fn battery_status(&self) -> u8;
    
    async fn vibrate(&self) -> Result<()>;
    
    async fn stop_vibrate(&self) -> Result<()>;
    
    async fn lock_device(&self) -> Result<()>;
    
    async fn unlock_device(&self) -> Result<()>;

}

pub trait IqosIluma {
    async fn update_brightness(&self, level: BrightnessLevel) -> Result<()>;
    
    async fn update_vibration_settings(&self, settings: VibrationSettings) -> Result<()>;
    
    async fn update_smartgesture(&self, enable: bool) -> Result<()>;
    
    async fn update_autostart(&self, enable: bool) -> Result<()>;
    
    async fn update_flexpuff(&self, enable: bool) -> Result<()>;
}