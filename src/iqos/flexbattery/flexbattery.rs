use crate::iqos::error::{IQOSError, Result};
use std::fmt;

pub const LOAD_FLEXBATTERY_SIGNAL: [u8; 5] = [0x00, 0xC9, 0x00, 0x25, 0xFB];
// Alternative signal format for pause mode loading
pub const LOAD_PAUSEMODE_SIGNAL: [u8; 9] = [0x00, 0xC9, 0x07, 0x24, 0x02, 0x00, 0x00, 0x00, 0x18];

pub const FLEXBATTERY_ECO_SIGNALS: [&[u8]; 2] = [
    &[0x00, 0xc9, 0x44, 0x25, 0x01, 0x00, 0x00, 0x00, 0x4D],
    &[0x00, 0xc9, 0x00, 0x25, 0xfb],
];

pub const FLEXBATTERY_PERFORMANCE_SIGNALS: [&[u8]; 2] = [
    &[0x00, 0xc9, 0x44, 0x25, 0x00, 0x00, 0x00, 0x00, 0x5B],
    &[0x00, 0xc9, 0x00, 0x25, 0xfb],
];

pub const PAUSEMODE_DISABLE_SIGNALS: [&[u8]; 2] = [
    &[0x00, 0xC9, 0x47, 0x24, 0x02, 0x00, 0x00, 0x00, 0x6E],
    &[0x00, 0xC9, 0x07, 0x24, 0x02, 0x00, 0x00, 0x00, 0x18],
];

pub const PAUSEMODE_ENABLE_SIGNALS: [&[u8]; 2] = [
    &[0x00, 0xC9, 0x47, 0x24, 0x02, 0x01, 0x00, 0x00, 0x05],
    &[0x00, 0xC9, 0x07, 0x24, 0x02, 0x00, 0x00, 0x00, 0x18],
];

pub type Pausemode = bool;

#[derive(Default, Clone, Debug)]
pub enum FlexbatteryMode {
    #[default]
    Performance,
    Eco,
}

impl FlexbatteryMode {
    pub fn to_bytes(&self) -> Vec<u8> {
        match self {
            FlexbatteryMode::Eco => FLEXBATTERY_ECO_SIGNALS[0].to_vec(),
            FlexbatteryMode::Performance => FLEXBATTERY_PERFORMANCE_SIGNALS[0].to_vec(),
        }
    }
}

#[derive(Default)]
pub struct FlexBattery {
    mode: FlexbatteryMode,
    is_pause_mode: Option<Pausemode>,
}

impl FlexBattery {
    pub fn new(mode: FlexbatteryMode) -> Self {
        Self { mode, is_pause_mode: None }
    }

    pub fn is_performance(&self) -> bool {
        matches!(self.mode, FlexbatteryMode::Performance)
    }

    pub fn is_pausemode(&self) -> Option<Pausemode> {
        self.is_pause_mode
    }

    pub fn mode(&self) -> &FlexbatteryMode {
        &self.mode
    }

    pub fn update_mode(& mut self, new: &FlexbatteryMode) {
        self.mode = new.clone();
    }

    pub fn update_pause_mode(&mut self, new: Pausemode) {
        self.is_pause_mode = Some(new);
    }

    pub fn build(&self) -> Vec<u8> {
        match self.mode {
            FlexbatteryMode::Eco => FLEXBATTERY_ECO_SIGNALS[0].to_vec(),
            FlexbatteryMode::Performance => FLEXBATTERY_PERFORMANCE_SIGNALS[0].to_vec(),
        }
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<FlexbatteryMode> {
        if bytes.len() < 9 {
            return Err(IQOSError::ConfigurationError("Invalid flexbattery mode data".to_string()));
        }

        if bytes[0] != 0x00 || bytes[1] != 0x08 || bytes[2] != 0x84 || bytes[3] != 0x25 {
            return Err(IQOSError::ConfigurationError("Invalid flexbattery mode header".to_string()));
        }
        let flag = bytes[4];
        match flag {
            0x00 => Ok(FlexbatteryMode::Performance),
            0x01 => Ok(FlexbatteryMode::Eco),
            _ => Err(IQOSError::ConfigurationError("Unknown flexbattery mode".to_string())),
        }
    }

    pub fn pausemode_from_bytes(bytes: &[u8]) -> Result<Pausemode> {
        if bytes.len() < 9 {
            return Err(IQOSError::ConfigurationError("Invalid pause mode data".to_string()));
        }

        if bytes[0] != 0x00 || bytes[1] != 0x08 || bytes[2] != 0x87 || bytes[3] != 0x24 {
            return Err(IQOSError::ConfigurationError("Invalid pause mode header".to_string()));
        }
        let flag = bytes[5];
        match flag {
            0x00 => Ok(false),
            0x01 => Ok(true),
            _ => Err(IQOSError::ConfigurationError("Unknown pause mode".to_string())),
        }
    }

    pub fn pausemode_to_bytes(pause_mode: Pausemode) -> Vec<u8> {
        if pause_mode {
            PAUSEMODE_ENABLE_SIGNALS[0].to_vec()
        } else {
            PAUSEMODE_DISABLE_SIGNALS[0].to_vec()
        }
    }

    pub fn from_args(args: &[&str]) -> Result<FlexBattery> {
        if args.is_empty() {
            return Err(IQOSError::ConfigurationError("No arguments provided".to_string()));
        }
        let mut flexbattery_mode = FlexbatteryMode::default();
        let mut pause_mode = None;

        let mut i = 0;
        while i < args.len() {
            let arg = args[i].to_lowercase();

            match arg.as_str() {
                "eco" => {
                    flexbattery_mode = FlexbatteryMode::Eco;
                    i += 1;
                },
                "performance" => {
                    flexbattery_mode = FlexbatteryMode::Performance;
                    i += 1;
                },
                "pausemode" => {
                    if i + 1 >= args.len() {
                        return Err(IQOSError::ConfigurationError("'on' or 'off' must follow the pausemode argument".to_string()));
                    }
                    
                    let pause_arg = args[i + 1].to_lowercase();
                    match pause_arg.as_str() {
                        "on" => pause_mode = Some(true),
                        "off" => pause_mode = Some(false),
                        _ => return Err(IQOSError::ConfigurationError("The pausemode value must be either 'on' or 'off'".to_string())),
                    }
                    
                    i += 2;
                },
                _ => {
                    return Err(IQOSError::ConfigurationError(format!("Unknown argument: {}", arg)));
                }
            }
        }
        
        Ok(FlexBattery {
            mode: flexbattery_mode,
            is_pause_mode: pause_mode,
        })
    }
}

impl fmt::Display for FlexbatteryMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FlexbatteryMode::Eco => write!(f, "Eco"),
            FlexbatteryMode::Performance => write!(f, "Performance"),
        }
    }
}

impl fmt::Display for FlexBattery {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Flexbattery: {}", self.mode)?;
        if let Some(pause_mode) = self.is_pause_mode {
            write!(f, "\nPause mode: {}", if pause_mode { "On" } else { "Off" })?;
        }
        Ok(())
    }
}

// Deprecated implementation, kept for reference
// impl std::fmt::Display for FlexBattery {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         if self.is_performance() {
//             write!(f, "FlexBattery Mode: {}\n Pause Mode: {}", self.mode, self.is_pause_mode.map_or("Off".to_string(), |p| if p { "On".to_string() } else { "Off".to_string() }))
//         } else {
//             write!(f, "FlexBattery Mode: {}", self.mode)
//         }
//     }
// }