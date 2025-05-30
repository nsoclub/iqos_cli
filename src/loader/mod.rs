use anyhow::Result;
use crate::iqos::IqosBle;

mod iqoshelper;
mod parser;

pub use parser::{IQOSConsole, CommandSystem, run_console};
