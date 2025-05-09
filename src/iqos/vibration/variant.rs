use crate::iqos::error::Result;
use super::VibrationSettings;


pub trait VibrationBehavior {
    fn checksum(&self, byte: &u16) -> u8;

    fn from_args(args: &[&str]) -> Result<VibrationSettings>;

    fn from_bytes(bytes: &[u8]) -> Result<VibrationSettings>;

    fn build(&self) -> Vec<Vec<u8>>;
}

pub trait IlumaVibrationBehavior: VibrationBehavior {
    fn from_args_with_charge_start(args: &[&str]) -> Result<VibrationSettings>;

    fn from_bytes(bytes: &[u8]) -> Result<VibrationSettings>;

    fn build(&self) -> Vec<Vec<u8>>;
}