pub const LOAD_FLEXBATTERY_SIGNAL: [u8; 5] = [0x00, 0xC9, 0x00, 0x25, 0xFB];
pub const LOAD_PAUSEMODE_SIGNAL: [u8; 9] = [0x00, 0xC9, 0x07, 0x24, 0x01, 0x00, 0x00, 0x00, 0x22];

pub const FLEXBATTERY_ECO_SIGNALS: [[u8]; 2] = [
    [0x00, 0xc9, 0x44, 0x25, 0x01, 0x00, 0x00, 0x00, 0x4D],
    [0x00, 0xc9, 0x00, 0x25, 0xfb],
];

type Pausemode = bool;

enum FlexbatteryMode {
    Eco,
    Performance,
}

struct Flexbattery {
    mode: FlexbatteryMode,
    is_pause_mode: Option<Pausemode>,
}

impl Flexbattery {
    pub fn new(mode: FlexbatteryMode) -> Self {
        Self { mode, is_pause_mode: None }
    }

    pub fn mode(&self, new: &FlexbatteryMode) {
        self.mode = new.clone();
    }

    pub fn build(&self) -> Vec<u8> {
        match self.mode {
            FlexbatteryMode::Eco => FLEXBATTERY_ECO_SIGNALS[0].to_vec(),
            FlexbatteryMode::Performance => FLEXBATTERY_PERFOMANCE_SIGNALS[1].to_vec(),
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

        if bytes[0] != 0x00 || bytes[1] != 0x08 || bytes[2] != 0x87 || bytes[3] != 0x24 || bytes[4] != 0x02 {
            return Err(IQOSError::ConfigurationError("Invalid pause mode header".to_string()));
        }
        let flag = bytes[5];
        match flag {
            0x00 => Ok(false),
            0x01 => Ok(true),
            _ => Err(IQOSError::ConfigurationError("Unknown pause mode".to_string())),
        }
    }
}