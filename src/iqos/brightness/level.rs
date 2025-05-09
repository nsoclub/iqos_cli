use crate::iqos::error::{IQOSError, Result};

pub const BRIGHTNESS_HIGH_SIGNAL: [&[u8]; 3] = [
    &[0x00, 0xc0, 0x46, 0x23, 0x64, 0x00, 0x00, 0x00, 0x4f],
    &[0x00, 0xc0, 0x02, 0x23, 0xc3],
    &[0x00, 0xc9, 0x44, 0x24, 0x64, 0x00, 0x00, 0x00, 0x34],

];
pub const BRIGHTNESS_LOW_SIGNAL: [&[u8]; 3] = [
    &[0x00, 0xc0, 0x46, 0x23, 0x1e, 0x00, 0x00, 0x00, 0xe1],
    &[0x00, 0xc0, 0x02, 0x23, 0xc3],
    &[0x00, 0xc9, 0x44, 0x24, 0x1e, 0x00, 0x00, 0x00, 0x9a],
];

pub const LOAD_BRIGHTNESS_SIGNAL: [u8; 5] = [0x00, 0xc0, 0x02, 0x23, 0xC3];

#[derive(Debug, Clone, Copy)]
pub enum BrightnessLevel {
    High,
    Low,
}

impl BrightnessLevel {
    pub fn from_bytes(bytes: &[u8]) -> Result<Self> {
        if bytes.len() < 9 {
            return Err(IQOSError::ConfigurationError("Invalid brightness level data".to_string()));
        }

        if bytes[0] != 0x00 || bytes[1] != 0xC0 || bytes[2] != 0x86 || bytes[3] != 0x23 {
            return Err(IQOSError::ConfigurationError("Invalid brightness level header".to_string()));
        }

        let flag = bytes[4];
        match flag {
            0x64 => Ok(BrightnessLevel::High),
            0x1e => Ok(BrightnessLevel::Low),
            _ => Err(IQOSError::ConfigurationError("Unknown brightness level".to_string())),
        }
    }
}

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
        let brightness_level = match self {
            BrightnessLevel::High => "high",
            BrightnessLevel::Low => "low",
        };
        write!(f, "\nBrightness Level: {}\n", brightness_level)
    }
}