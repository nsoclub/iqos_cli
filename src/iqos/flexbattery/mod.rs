const FLEXBATTERY_ECO_SIGNALS: [[u8]; 2] = [
    [0x00, 0xc9, 0x44, 0x25, 0x01, 0x00, 0x00, 0x00, 0x4D],
    [0x00, 0xc9, 0x00, 0x25, 0xfb],
];

enum FlexbatteryMode {
    Eco,
    Performance,
}

struct Flexbattery {
    mode: FlexbatteryMode,
}

impl Flexbattery {
    fn new(mode: FlexbatteryMode) -> Self {
        Self { mode }
    }

    fn build(&self) -> Vec<u8> {
        match self.mode {
            FlexbatteryMode::Eco => FLEXBATTERY_ECO_SIGNALS[0].to_vec(),
            FlexbatteryMode::Performance => FLEXBATTERY_ECO_SIGNALS[1].to_vec(),
        }
    }
}