use crate::iqos::error::{IQOSError, Result};

use super::device::IqosIluma;
use super::iqos::IqosBle;
use btleplug::api::{Peripheral as _, WriteType};

pub struct IlumaSpecific {
    holder_product_number: String,
    firmware_version: String,
}

impl IlumaSpecific {
    pub fn new(holder_product_number: String, firmware_version: String) -> Self {
        Self {
            holder_product_number,
            firmware_version,
        }
    }

    pub fn holder_product_number(&self) -> &str {
        &self.holder_product_number
    }
}

#[derive(Debug, Clone, Copy)]
pub enum BrightnessLevel {
    High,
    Low,
}

pub const BRIGHTNESS_HIGH_SIGNAL_FIRST: [u8; 9] = [0x00, 0xc0, 0x46, 0x23, 0x64, 0x00, 0x00, 0x00, 0x4f];
pub const BRIGHTNESS_HIGH_SIGNAL_SECOND: [u8; 5] = [0x00, 0xc0, 0x02, 0x23, 0xc3];
pub const BRIGHTNESS_HIGH_SIGNAL_THIRD: [u8; 9] = [0x00, 0xc9, 0x44, 0x24, 0x64, 0x00, 0x00, 0x00, 0x34];
pub const BRIGHTNESS_LOW_SIGNAL_FIRST: [u8; 9] = [0x00, 0xc0, 0x46, 0x23, 0x1e, 0x00, 0x00, 0x00, 0xe1];
pub const BRIGHTNESS_LOW_SIGNAL_SECOND: [u8; 5] = [0x00, 0xc0, 0x02, 0x23, 0xc3];
pub const BRIGHTNESS_LOW_SIGNAL_THIRD: [u8; 9] = [0x00, 0xc9, 0x44, 0x24, 0x1e, 0x00, 0x00, 0x00, 0x9a];

pub const FLEXPUFF_ENABLE_SIGNAL: [u8; 9] = [0x00, 0xd2, 0x45, 0x22, 0x03, 0x01, 0x00, 0x00, 0x0A];
pub const FLEXPUFF_DISABLE_SIGNAL: [u8; 9] = [0x00, 0xd2, 0x45, 0x22, 0x03, 0x00, 0x00, 0x00, 0x0A];
pub const AUTOSTART_ENABLE_SIGNAL: [u8; 9] = [0x00, 0xc9, 0x47, 0x24, 0x01, 0x01, 0x00, 0x00, 0x3f];
pub const AUTOSTART_DISABLE_SIGNAL: [u8; 9] = [0x00, 0xc9, 0x47, 0x24, 0x01, 0x00, 0x00, 0x00, 0x54];

pub const SMARTGESTURE_ENABLE_SIGNAL: [u8; 9] = [0x00, 0xc9, 0x47, 0x24, 0x04, 0x01, 0x00, 0x00, 0x3c];
pub const SMARTGESTURE_DISABLE_SIGNAL: [u8; 9] = [0x00, 0xc9, 0x47, 0x24, 0x04, 0x00, 0x00, 0x00, 0x57];

pub static WHEN_CHARGING_START_ON_SIGNALS: [&[u8]; 7] = [
    &[0x01, 0xC9, 0x4F, 0x04, 0x72, 0x04, 0x00, 0xFF, 0xFF, 0xFF, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00],
    &[0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x06],
    &[0x01, 0xC9, 0x4F, 0x04, 0x72, 0x05, 0x00, 0xFF, 0xFF, 0xFF, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00],
    &[0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x72],
    &[0x00, 0xC9, 0x47, 0x04, 0x00, 0xFF, 0xFF, 0x00, 0xDA],
    &[0x00, 0xC9, 0x07, 0x04, 0x04, 0x00, 0x00, 0x00, 0x08],
    &[0x00, 0xC9, 0x07, 0x04, 0x05, 0x00, 0x00, 0x00, 0x1E],
];
pub static WHEN_CHARGING_START_OFF_SIGNALS: [&[u8]; 7] = [
    &[0x01, 0xC9, 0x4F, 0x04, 0x64, 0x04, 0x00, 0xFF, 0xFF, 0xFF, 0x09, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00],
    &[0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x0c],
    &[0x01, 0xC9, 0x4F, 0x04, 0x4d, 0x05, 0x00, 0xFF, 0xFF, 0xFF, 0x09, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00],
    &[0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x78],
    &[0x00, 0xC9, 0x47, 0x04, 0x00, 0xFF, 0xFF, 0x00, 0xDA],
    &[0x00, 0xC9, 0x07, 0x04, 0x04, 0x00, 0x00, 0x00, 0x08],
    &[0x00, 0xC9, 0x07, 0x04, 0x05, 0x00, 0x00, 0x00, 0x1E],
];
pub const WHEN_STARTING_TO_USE_SIGNAL: u16 = 0x1000;
pub const WHEN_HEATING_START_SIGNAL: u16 = 0x0100;
pub const WHEN_MANUALLY_TERMINATED_SIGNAL: u16 = 0x0010;
pub const WHEN_PUFF_END_SIGNAL: u16 = 0x0001;

#[derive(Debug)]
pub struct NotIlumaError;

impl std::fmt::Display for NotIlumaError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "This device is not an IQOS ILUMA model")
    }
}

impl std::error::Error for NotIlumaError {}

impl std::str::FromStr for BrightnessLevel {
    type Err = IQOSError;

    fn from_str(s: &str) -> Result<Self> {
        match s.to_lowercase().as_str() {
            "high" => Ok(BrightnessLevel::High),
            "low" => Ok(BrightnessLevel::Low),
            _ => Err(IQOSError::ConfigurationError("Invalid brightness level".to_string())),
        }
    }
}

impl std::fmt::Display for BrightnessLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BrightnessLevel::High => write!(f, "high"),
            BrightnessLevel::Low => write!(f, "low"),
        }
    }
}

pub struct VibrationSettings {
    pub when_charging_start: bool,
    pub when_heating_start: bool,
    pub when_starting_to_use: bool,
    pub when_puff_end: bool,
    pub when_manually_terminated: bool,
}

impl VibrationSettings {
    pub fn new(
        when_charging_start: bool,
        when_heating_start: bool,
        when_starting_to_use: bool,
        when_puff_end: bool,
        when_manually_terminated: bool,
    ) -> Self {
        Self {
            when_charging_start,
            when_heating_start,
            when_starting_to_use,
            when_puff_end,
            when_manually_terminated,
        }
    }

    fn checksum(&self, data: &u16) -> u8 {
        let mut checksum: u8 = 0x77;

        if (data & 0x0001) != 0 {
            checksum ^= 0x07;
        }
        if (data & 0x0010) != 0 {
            checksum ^= 0x70;
        }
        if (data & 0x0100) != 0 {
            checksum ^= 0x15;
        }
        if (data & 0x1000) != 0 {
            checksum ^= 0x57;
        }

        checksum
    } 

    pub fn build(&self) -> Vec<Vec<u8>> {
        let mut ret = vec![];
        let mut reg = 0u16;

        // 充電開始時の設定 - これは特別なシグナルを使用
        if self.when_charging_start {
            ret.extend(
                WHEN_CHARGING_START_ON_SIGNALS
                    .iter()
                    .map(|&signal| signal.to_vec())
            );
        } else {
            ret.extend(
                WHEN_CHARGING_START_OFF_SIGNALS
                    .iter()
                    .map(|&signal| signal.to_vec())
            );
        }

        // 他の設定を反映
        if self.when_heating_start {
            reg |= WHEN_HEATING_START_SIGNAL;
        }
        if self.when_starting_to_use {
            reg |= WHEN_STARTING_TO_USE_SIGNAL;
        }
        if self.when_puff_end {
            reg |= WHEN_PUFF_END_SIGNAL;
        }
        if self.when_manually_terminated {
            reg |= WHEN_MANUALLY_TERMINATED_SIGNAL;
        }

        // すべての設定がオフかどうかをチェック（充電開始は除く）
        let all_other_settings_off = !self.when_heating_start && 
                                    !self.when_starting_to_use && 
                                    !self.when_puff_end && 
                                    !self.when_manually_terminated;

        // すべての設定がオフで、かつ充電開始もオフの場合は特別なケース
        if all_other_settings_off && !self.when_charging_start {
            println!("hey guys");
            let mut signal = vec![0x00, 0xC9, 0x44, 0x23, 0x10, 0x00];
            signal.push(0x00);  // reg上位バイト
            signal.push(0x00);  // reg下位バイト
            signal.push(self.checksum(&0x0000));
            ret.push(signal);
            return ret;
        }

        // メインのシグナルを構築
        // 充電開始がオンでも、他の設定のレジスタ値を適用する
        let mut signal = vec![0x00, 0xC9, 0x44, 0x23, 0x10, 0x00];
        signal.push((reg >> 8) as u8);  // 上位バイト
        signal.push((reg & 0xff) as u8); // 下位バイト
        signal.push(self.checksum(&reg));

        ret.push(signal);
        ret
    }

    pub fn parse_from_args(args: &[&str]) -> Self {
        let mut when_charging_start = false;
        let mut when_heating_start = false;
        let mut when_starting_to_use = false;
        let mut when_puff_end = false;
        let mut when_manually_terminated = false;
        
        // Parse args in pairs (option + value)
        let mut i = 0;
        while i < args.len() - 1 {
            match args[i] {
                "charge" => {
                    when_charging_start = args[i+1] == "on";
                },
                "heating" => {
                    when_heating_start = args[i+1] == "on";
                },
                "starting" => {
                    when_starting_to_use = args[i+1] == "on";
                },
                "terminated" => {
                    when_manually_terminated = args[i+1] == "on";
                },
                "puffend" => {
                    when_puff_end = args[i+1] == "on";
                },
                _ => {} // Ignore unknown options
            }
            i += 2;
        }
        
        Self {
            when_charging_start,
            when_heating_start,
            when_starting_to_use,
            when_puff_end,
            when_manually_terminated,
        }
    }
}

impl IqosIluma for IqosBle {
    async fn update_brightness(&self, level: BrightnessLevel) -> Result<()> {
        if !self.is_iluma() {
            return Err(IQOSError::IncompatibleModelError);
        }

        let signals: Vec<Vec<u8>> = match level {
            BrightnessLevel::High => vec!(
                BRIGHTNESS_HIGH_SIGNAL_FIRST.to_vec(),
                BRIGHTNESS_HIGH_SIGNAL_SECOND.to_vec(),
                BRIGHTNESS_HIGH_SIGNAL_THIRD.to_vec(),
            ),
            BrightnessLevel::Low => vec!(
                BRIGHTNESS_LOW_SIGNAL_FIRST.to_vec(),
                BRIGHTNESS_LOW_SIGNAL_SECOND.to_vec(),
                BRIGHTNESS_LOW_SIGNAL_THIRD.to_vec(),
            ),
        };

        let peripheral = self.peripheral();
        let characteristic = self.scp_control_characteristic();

        for signal in signals {
            peripheral.write(
                characteristic,
                &signal,
                WriteType::WithResponse,
            ).await.map_err(IQOSError::BleError)?;
        }

        Ok(())
    }

    async fn update_vibration_settings(&self, settings: VibrationSettings) -> Result<()> {
        if !self.is_iluma() {
            return Err(IQOSError::IncompatibleModelError);
        }

        let signals = settings.build();
        for (i, signal) in signals.iter().enumerate() {
            let hex_string = signal.iter()
                .map(|byte| format!("{:02X}", byte))
                .collect::<Vec<String>>()
                .join(" ");
            println!("  Signal {}: {}", i, hex_string);
        }
        // return Ok(());
        
        for signal in signals {
            self.send_command(signal).await?;
        }

        Ok(())
    }

    async fn update_smartgesture(&self, enable: bool) -> Result<()> {
        if !self.is_iluma() {
            return Err(IQOSError::IncompatibleModelError);
        }

        let signal = if enable {
            SMARTGESTURE_ENABLE_SIGNAL
        } else {
            SMARTGESTURE_DISABLE_SIGNAL
        };

        let peripheral = self.peripheral();
        let characteristic = self.scp_control_characteristic();
        
        peripheral.write(
            characteristic,
            &signal,
            WriteType::WithResponse,
        ).await.map_err(IQOSError::BleError)?;

        Ok(())
    }

    async fn update_autostart(&self, enable: bool) -> Result<()> {
        if !self.is_iluma() {
            return Err(IQOSError::IncompatibleModelError);
        }

        let signal = if enable {
            AUTOSTART_ENABLE_SIGNAL
        } else {
            AUTOSTART_DISABLE_SIGNAL
        };

        let peripheral = self.peripheral();
        let characteristic = self.scp_control_characteristic();
        
        peripheral.write(
            characteristic,
            &signal,
            WriteType::WithResponse,
        ).await.map_err(IQOSError::BleError)?;

        Ok(())
    }

    async fn update_flexpuff(&self, enable: bool) -> Result<()> {
        if !self.is_iluma() {
            return Err(IQOSError::IncompatibleModelError);
        }

        let signal = if enable {
            FLEXPUFF_ENABLE_SIGNAL
        } else {
            FLEXPUFF_DISABLE_SIGNAL
        };

        let peripheral = self.peripheral();
        let characteristic = self.scp_control_characteristic();
        
        peripheral.write(
            characteristic,
            &signal,
            WriteType::WithResponse,
        ).await.map_err(IQOSError::BleError)?;

        Ok(())
    }
}