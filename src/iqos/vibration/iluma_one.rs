use crate::iqos::error::{IQOSError, Result};
use super::variant::VibrationBehavior;
use super::settings::{VibrationSettings, WHEN_HEATING_START_SIGNAL, WHEN_STARTING_TO_USE_SIGNAL, WHEN_PUFF_END_SIGNAL, WHEN_MANUALLY_TERMINATED_SIGNAL};

impl VibrationBehavior for VibrationSettings {
    fn checksum(&self, byte: &u16) -> u8 {
        let mut checksum: u8 = 0x77;

        if (byte & 0x0001) != 0 {
            checksum ^= 0x07;
        }
        if (byte & 0x0010) != 0 {
            checksum ^= 0x70;
        }
        if (byte & 0x0100) != 0 {
            checksum ^= 0x15;
        }
        if (byte & 0x1000) != 0 {
            checksum ^= 0x57;
        }

        checksum
    }

    fn build(&self) -> Vec<Vec<u8>> {
        let mut ret = vec![];
        let mut reg = 0u16;

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

        if all_other_settings_off {
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

    fn from_args(args: &[&str]) -> Result<VibrationSettings> {
        let mut when_heating_start = false;
        let mut when_starting_to_use = false;
        let mut when_puff_end = false;
        let mut when_manually_terminated = false;
        
        let mut i = 0;
        while i < args.len() - 1 {
            match args[i] {
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
            iluma_and_higher: None,
            when_heating_start,
            when_starting_to_use,
            when_puff_end,
            when_manually_terminated,
        })
    }

    fn from_bytes(bytes: &[u8]) -> Result<Self> {
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
