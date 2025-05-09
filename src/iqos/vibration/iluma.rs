use crate::iqos::error::{IQOSError, Result};
use super::variant::{VibrationBehavior, IlumaVibrationBehavior};
use super::settings::{VibrationSettings, WHEN_HEATING_START_SIGNAL, WHEN_STARTING_TO_USE_SIGNAL, WHEN_PUFF_END_SIGNAL, WHEN_MANUALLY_TERMINATED_SIGNAL};

pub const WHEN_CHARGING_START_ON_SIGNALS: [&[u8]; 7] = [
    &[0x01, 0xC9, 0x4F, 0x04, 0x72, 0x04, 0x00, 0xFF, 0xFF, 0xFF, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00],
    &[0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x06],
    &[0x01, 0xC9, 0x4F, 0x04, 0x72, 0x05, 0x00, 0xFF, 0xFF, 0xFF, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00],
    &[0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x72],
    &[0x00, 0xC9, 0x47, 0x04, 0x00, 0xFF, 0xFF, 0x00, 0xDA],
    &[0x00, 0xC9, 0x07, 0x04, 0x04, 0x00, 0x00, 0x00, 0x08],
    &[0x00, 0xC9, 0x07, 0x04, 0x05, 0x00, 0x00, 0x00, 0x1E],
];

pub const WHEN_CHARGING_START_OFF_SIGNALS: [&[u8]; 7] = [
    &[0x01, 0xC9, 0x4F, 0x04, 0x64, 0x04, 0x00, 0xFF, 0xFF, 0xFF, 0x09, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00],
    &[0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x0c],
    &[0x01, 0xC9, 0x4F, 0x04, 0x4d, 0x05, 0x00, 0xFF, 0xFF, 0xFF, 0x09, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00],
    &[0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x78],
    &[0x00, 0xC9, 0x47, 0x04, 0x00, 0xFF, 0xFF, 0x00, 0xDA],
    &[0x00, 0xC9, 0x07, 0x04, 0x04, 0x00, 0x00, 0x00, 0x08],
    &[0x00, 0xC9, 0x07, 0x04, 0x05, 0x00, 0x00, 0x00, 0x1E],
];
pub struct IlumaVibration {
    pub when_charging_start: bool,
}

impl IlumaVibration {
    pub fn when_charge_start(&self) -> bool {
        self.when_charging_start
    }
}

impl IlumaVibrationBehavior for VibrationSettings {
    fn build(&self) -> Vec<Vec<u8>> {
        let mut ret = vec![];
        let mut reg = 0u16;

        let when_charge_start = self.iluma_and_higher.as_ref().unwrap().when_charge_start();
        
        // 充電開始時の設定を追加
        if when_charge_start {
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

        let all_other_settings_off = !self.when_heating_start && 
                                           !self.when_starting_to_use && 
                                           !self.when_puff_end && 
                                           !self.when_manually_terminated;

        if all_other_settings_off && !when_charge_start {
            let mut signal = vec![0x00, 0xC9, 0x44, 0x23, 0x10, 0x00];
            signal.push(0x00);
            signal.push(0x00);
            signal.push(self.checksum(&0x0000));
            ret.push(signal);
            return ret;
        }

        let mut signal = vec![0x00, 0xC9, 0x44, 0x23, 0x10, 0x00];
        signal.push((reg >> 8) as u8);
        signal.push((reg & 0xff) as u8);
        signal.push(self.checksum(&reg));

        ret.push(signal);
        ret
    }

    fn from_args_with_charge_start(args: &[&str]) -> Result<VibrationSettings> {
        let mut when_charging_start = false;
        let mut when_heating_start = false;
        let mut when_starting_to_use = false;
        let mut when_puff_end = false;
        let mut when_manually_terminated = false;
        
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
                _ => {}
            }
            i += 2;
        }
        
        Ok(Self {
            iluma_and_higher: Some(IlumaVibration { when_charging_start }),
            when_heating_start,
            when_starting_to_use,
            when_puff_end,
            when_manually_terminated,
        })
    }

    fn from_bytes(bytes: &[u8]) -> Result<VibrationSettings> {
        if bytes.len() < 9 {
            return Err(IQOSError::ConfigurationError("Data too short for vibration settings".to_string()));
        }
        
        if bytes[0] != 0x00 || bytes[1] != 0x08 || bytes[2] != 0x84 || bytes[3] != 0x23 || bytes[4] != 0x10 {
            return Err(IQOSError::ConfigurationError("Invalid header for vibration settings".to_string()));
        }

        // Parse vibration settings from bytes 8 and 9
        // Byte 8 (index 7) contains "heat" (bit 0) and "use" (bit 4) settings
        // Byte 9 (index 8) contains "end" (bit 0) and "terminated" (bit 4) settings
        let heat_use_byte = bytes[6];
        let end_terminated_byte = bytes[7];
        
        let when_heating_start = (heat_use_byte & 0x01) != 0;
        let when_starting_to_use = (heat_use_byte & 0x10) != 0;
        let when_puff_end = (end_terminated_byte & 0x01) != 0;
        let when_manually_terminated = (end_terminated_byte & 0x10) != 0;
        
        Ok(Self {
            iluma_and_higher: None,
            when_heating_start,
            when_starting_to_use,
            when_puff_end,
            when_manually_terminated,
        })
    }
} 