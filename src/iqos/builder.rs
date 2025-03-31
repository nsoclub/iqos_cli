use super::device::IQOS;
use super::error::{IQOSError, Result};
use btleplug::platform::Peripheral;

pub struct IQOSBuilder {
    peripheral: Option<Peripheral>,
    modelnumber: Option<String>,
    serialnumber: Option<String>,
    softwarerevision: Option<String>,
    manufacturername: Option<String>,
}

impl IQOSBuilder {
    pub fn new() -> Self {
        Self {
            peripheral: None,
            modelnumber: None,
            serialnumber: None,
            softwarerevision: None,
            manufacturername: None,
        }
    }

    pub fn with_peripheral(mut self, peripheral: Peripheral) -> Self {
        self.peripheral = Some(peripheral);
        self
    }

    pub fn with_model_number(mut self, modelnumber: impl Into<String>) -> Self {
        self.modelnumber = Some(modelnumber.into());
        self
    }

    pub fn with_serial_number(mut self, serialnumber: impl Into<String>) -> Self {
        self.serialnumber = Some(serialnumber.into());
        self
    }

    pub fn with_software_revision(mut self, softwarerevision: impl Into<String>) -> Self {
        self.softwarerevision = Some(softwarerevision.into());
        self
    }

    pub fn with_manufacturer_name(mut self, manufacturername: impl Into<String>) -> Self {
        self.manufacturername = Some(manufacturername.into());
        self
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
        );

        Ok(iqos)
    }
}

impl Default for IQOSBuilder {
    fn default() -> Self {
        Self::new()
    }
} 