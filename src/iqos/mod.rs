mod builder;
mod device;
mod error;

use uuid::{uuid, Uuid};

pub use builder::IQOSBuilder;
pub use device::IQOS;

// Service UUIDs
pub const DEVICE_INFO_SERVICE_UUID: Uuid = uuid!("0000180a-0000-1000-8000-00805f9b34fb");
pub const CORE_SERVICE_UUID: Uuid = uuid!("daebb240-b041-11e4-9e45-0002a5d5c51b");

// Characteristic UUIDs
pub const MODEL_NUMBER_CHAR_UUID: &str = "00002a24";
pub const SERIAL_NUMBER_CHAR_UUID: &str = "00002a25";
pub const SOFTWARE_REVISION_CHAR_UUID: &str = "00002a28";
pub const MANUFACTURER_NAME_CHAR_UUID: &str = "00002a29";
pub const BATTERY_CHARACTERISTIC_UUID: Uuid = uuid!("f8a54120-b041-11e4-9be7-0002a5d5c51b");
pub const SCP_CONTROL_CHARACTERISTIC_UUID: Uuid = uuid!("e16c6e20-b041-11e4-a4c3-0002a5d5c51b");