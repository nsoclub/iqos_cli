use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

use anyhow::Result;
use tokio::sync::Mutex;

use crate::iqos::IqosBle;

/// Command function type - represents a function that can be executed as a CLI command
pub type CommandFn = Box<dyn Fn(Arc<Mutex<IqosBle>>, Vec<String>) -> Pin<Box<dyn Future<Output = Result<()>> + Send>> + Send + Sync>;

/// Type alias for command registry
pub type CommandRegistry = HashMap<String, CommandFn>;

/// Command registration function signature
pub type CommandRegistrationFn = fn(&mut CommandRegistry);

/// Basic description of a command - useful for help text and documentation
pub struct CommandInfo {
    pub name: &'static str,
    pub description: &'static str,
    pub usage: &'static str,
    pub requires_iluma: bool,
    pub requires_iluma_i: bool,
}

impl CommandInfo {
    /// Create a new command info
    pub fn new(
        name: &'static str,
        description: &'static str,
        usage: &'static str,
        requires_iluma: bool,
        requires_iluma_i: bool,
    ) -> Self {
        Self {
            name,
            description,
            usage,
            requires_iluma,
            requires_iluma_i,
        }
    }
    
    /// Create command info from tuple format for backward compatibility
    pub fn from_tuple(info: (&'static str, &'static str, &'static str, bool)) -> Self {
        let (name, description, usage, requires_iluma) = info;
        Self {
            name,
            description,
            usage,
            requires_iluma,
            requires_iluma_i: false,
        }
    }
}
