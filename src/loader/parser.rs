use rustyline::error::ReadlineError;
use rustyline::{Config, DefaultEditor, Editor};
use anyhow::Result;
use tokio::sync::Mutex;

use std::collections::HashMap;
use std::pin::Pin;
use std::sync::Arc;
use std::future::Future;

use crate::loader::iqoshelper::IqosHelper;

use crate::iqos::vibration::IlumaVibrationBehavior;
use crate::iqos::{IqosBle, BrightnessLevel, IqosIluma, IqosIlumaI, Iqos, VibrationSettings, vibration::VibrationBehavior};
use crate::iqos::flexbattery::{FlexBattery, FlexbatteryMode};

// Command function type definition
type CommandFn = Box<dyn Fn(&IQOSConsole, Vec<String>) -> Pin<Box<dyn Future<Output = Result<()>> + Send>> + Send + Sync>;

#[derive(Clone)]
pub struct IQOSConsole {
    commands: Arc<Mutex<HashMap<String, CommandFn>>>,
    iqos: Arc<Mutex<IqosBle>>,
    running: bool,
}

impl IQOSConsole {
    pub fn new(iqos: IqosBle) -> Self {
        let console = Self {
            commands: Arc::new(Mutex::new(HashMap::new())),
            iqos: Arc::new(Mutex::new(iqos)),
            running: false,
        };
        
        let console_clone = console.clone();
        tokio::spawn(async move {
            console_clone.register_commands().await;
        });
        
        console
    }
    
    async fn register_commands(&self) {
        let mut commands = self.commands.lock().await;
        
        commands.insert("battery".to_string(), Box::new(|console: &IQOSConsole, _| {
            let iqos = console.iqos.clone();
            Box::pin(async move {
                let mut iqos = iqos.lock().await;
                iqos.reload_battery().await?;
                println!("Battery status: {}%", iqos.battery_status());
                Ok(())
            })
        }));

        commands.insert("brightness".to_string(), Box::new(|console: &IQOSConsole, args| {
            let iqos = console.iqos.clone();
            let args = args.clone();
            Box::pin(async move {
                let iqos = iqos.lock().await;
                match args.get(1).map(|s| s.parse::<BrightnessLevel>()) {
                    Some(Ok(level)) => {
                        if iqos.is_iluma() {
                            iqos.update_brightness(level).await?;
                            println!("Set brightness to {}", level);
                        } else {
                            println!("This device is not an ILUMA model");
                        }
                    },
                    Some(Err(e)) => println!("{}", e),
                    None => iqos.load_brightness().await.map(|level| {
                        println!("{}", level);
                    }).unwrap_or_else(|_| {
                        println!("This device is not an ILUMA model");
                    }),
                }
                Ok(())
            })
        }));

        commands.insert("lock".to_string(), Box::new(|console: &IQOSConsole, _| {
            let iqos = console.iqos.clone();
            Box::pin(async move {
                let iqos = iqos.lock().await;
                iqos.lock_device().await?;
                println!("Locked the IQOS");
                Ok(())
            })
        }));

        commands.insert("unlock".to_string(), Box::new(|console: &IQOSConsole, _| {
            let iqos = console.iqos.clone();
            Box::pin(async move {
                let iqos = iqos.lock().await;
                iqos.unlock_device().await?;
                println!("Unlocked the IQOS");
                Ok(())
            })
        }));
        
        commands.insert("findmyiqos".to_string(), Box::new(|console: &IQOSConsole, _| {
            let iqos = console.iqos.clone();
            let mut rl = DefaultEditor::new().unwrap();
            Box::pin(async move {
                let iqos = iqos.lock().await;
                println!("Find My IQOS...");
                iqos.vibrate().await?;
                let input = rl.readline("Press <Enter> to stop vibration");
                
                match input {
                    Ok(_) => {
                        iqos.stop_vibrate().await?;
                        print!("Vibration stopped.\n");
                    }
                    Err(_) => {
                        println!("Vibration stopped.");
                    }
                }

                Ok(())
            })
        }));

        commands.insert("flexbattery".to_string(), Box::new(|console: &IQOSConsole, args | {
            let iqos = console.iqos.clone();
            let args_cloned = args.clone();
            Box::pin(async move {
                let iqos = iqos.lock().await;
                let str_args: Vec<&str> = args_cloned.iter().map(|s| s.as_str()).collect();

                if args_cloned.len() == 1 {
                    // No arguments provided, show current flexbattery mode
                    if let Some(iluma_i) = iqos.as_iluma_i() {
                        match iluma_i.load_flexbattery().await {
                            Ok(flexbattery) => println!("\n{}\n", flexbattery),
                            Err(e) => println!("Error: {}", e),
                        }
                    } else {
                        println!("This device is not an ILUMA model");
                    }
                } else if args_cloned.len() >= 2 {
                    if let Some(iluma_i) = iqos.as_iluma_i() {
                        let fb = FlexBattery::from_args(&str_args[1..])?;
                        iluma_i.update_flexbattery(fb).await?;
                        println!("Flexbattery mode updated.");
                    } else {
                        println!("This device is not an ILUMA model");
                    }
                } else {
                    println!("Usage: flexbattery [performance|eco] | pause [on|off]");
                }
                Ok(())
            })
        }));

        commands.insert("flexpuff".to_string(), Box::new(|console: &IQOSConsole, args| {
            let iqos = console.iqos.clone();
            let args_cloned = args.clone();
            Box::pin(async move {
                let iqos = iqos.lock().await;
                match args_cloned.get(1).map(|s| s.as_str()) {
                    Some("status") => {
                        let status = iqos.load_flexpuff().await?;
                        println!("\nFlexpuff status: {}\n", status);
                    },
                    Some("enable") => {
                        let flexpuff = crate::iqos::flexpuff::Flexpuff::new(true);
                        let result = iqos.update_flexpuff(flexpuff).await;
                        match result {
                            Ok(_) => println!("Flexpuff enabled"),
                            Err(e) => println!("Error: {}", e),
                        }
                    },
                    Some("disable") => {
                        let flexpuff = crate::iqos::flexpuff::Flexpuff::new(false);
                        let result = iqos.update_flexpuff(flexpuff).await;
                        match result {
                            Ok(_) => println!("Flexpuff disabled"),
                            Err(e) => println!("Error: {}", e),
                        }
                    },
                    Some(opt) => println!("Invalid option: {}. Please specify 'enable' or 'disable'", opt),
                    None => println!("Usage: flexpuff [status|enable|disable]"),
                }
                Ok(())
            })
        }));
        
        commands.insert("smartgesture".to_string(), Box::new(|console: &IQOSConsole, args| {
            let iqos = console.iqos.clone();
            let args_cloned = args.clone();
            Box::pin(async move {
                let iqos = iqos.lock().await;
                if let Some(iluma) = iqos.as_iluma() {
                    match args_cloned.get(1).map(|s| s.as_str()) {
                        Some("enable") => {
                            let result = iluma.update_smartgesture(true).await;
                            match result {
                                Ok(_) => println!("Smart Gesture enabled"),
                                Err(e) => println!("Error: {}", e),
                            }
                        },
                        Some("disable") => {
                            let result = iluma.update_smartgesture(false).await;
                            match result {
                                Ok(_) => println!("Smart Gesture disabled"),
                                Err(e) => println!("Error: {}", e),
                            }
                        },
                        Some(opt) => println!("Invalid option: {}. Please specify 'enable' or 'disable'", opt),
                        None => println!("Usage: smartgesture [enable|disable]"),
                    }
                } else {
                    println!("This device is not an ILUMA model");
                }
                Ok(())
            })
        }));

        commands.insert("autostart".to_string(), Box::new(|console: &IQOSConsole, args| {
            let iqos = console.iqos.clone();
            let args_cloned = args.clone();
            Box::pin(async move {
                let iqos = iqos.lock().await;
                if let Some(arg) = args_cloned.get(1) {
                    match arg.to_lowercase() {
                        s if s == "on" || s == "enable" => {
                            let result = iqos.update_autostart(true).await;
                            match result {
                                Ok(_) => println!("Autostart enabled"),
                                Err(e) => println!("Error: {}", e),
                            }
                        },
                        s if s == "off" || s == "disable" => {
                            let result = iqos.update_autostart(false).await;
                            match result {
                                Ok(_) => println!("Autostart disabled"),
                                Err(e) => println!("Error: {}", e),
                            }
                        },
                        _ => println!("Usage: autostart [on|off]"),
                    }
                } else {
                    println!("Usage: autostart [on|off]");
                }
                Ok(())
            })
        }));

        commands.insert("vibration".to_string(), Box::new(|console: &IQOSConsole, args| {
            let iqos = console.iqos.clone();
            let args_cloned = args.clone();
            Box::pin(async move {
                let iqos = iqos.lock().await;
                let str_args: Vec<&str> = args_cloned.iter().map(|s| s.as_str()).collect();
                
                if str_args.len() >= 2 {
                    let param_args = &str_args[1..];
                    
                    if let Some(iluma) = iqos.as_iluma() {
                        let settings = VibrationSettings::from_args_with_charge_start(param_args)?;
                        iluma.update_iluma_vibration_settings(settings).await?;
                        println!("Vibration settings updated");
                    } else {
                        let settings = VibrationSettings::from_args(param_args)?;
                        iqos.update_vibration_settings(settings).await?;
                        println!("Vibration settings updated");
                    }
                } else if args_cloned.len() == 1 {
                    if let Some(iluma) = iqos.as_iluma() {
                        match iluma.load_iluma_vibration_settings().await {
                            Ok(settings) => {
                                println!("is iluma : {:?}", settings);
                                println!("{}", settings);
                            },
                            Err(e) => println!("Error: {}", e),
                        }
                    } else {
                        match iqos.load_vibration_settings().await {
                            Ok(settings) => println!("{}", settings),
                            Err(e) => println!("Error: {}", e),
                        }
                    }
                } else {
                    println!("Usage: vibration [charge|heating|starting|terminated|puffend] [on|off] ...");
                    println!("Example: vibration charge on heating on puffend on");
                    println!("Note: charge option is only available for ILUMA models");
                }
                Ok(())
            })
        }));
        
        commands.insert("help".to_string(), Box::new(|console: &IQOSConsole, _| {
            let iqos = console.iqos.clone();
            Box::pin(async move {
                let iqos = iqos.lock().await;
                let is_iluma = iqos.is_iluma();
                
                println!("Available commands:");
                
                // Common commands
                println!("  battery - Display battery status");
                println!("  lock | unlock - Lock or unlock the device");
                println!("  findmyiqos - Activate find my device feature");
                println!("  autostart [on|off] - Configure auto-start feature");
                
                // ILUMA model specific commands
                if is_iluma {
                    println!("\nILUMA model specific commands:");
                    println!("  brightness [high|low] - Set brightness");
                    println!("  smartgesture [enable|disable] - Configure Smart Gesture feature");
                    println!("  flexpuff [enable|disable] - Configure FlexPuff feature");
                    println!("  vibration [option on|off]... - Configure vibration settings");
                    println!("    Example: vibration charge on heating on puffend on");
                    println!("    Options: charge, heating, starting, terminated, puffend");
                }
                
                println!("\nOther commands:");
                println!("  info - Display device status");
                println!("  help - Display this help message");
                println!("  quit | exit - Exit the program");
                
                Ok(())
            })
        }));
        
        commands.insert("info".to_string(), Box::new(|console: &IQOSConsole, _| {
            let iqos = console.iqos.clone();
            Box::pin(async move {
                let iqos = iqos.lock().await;
                println!("\n{}\n", iqos);
                Ok(())
            })
        }));
    }
    
    async fn execute(&self, line: &str) -> Result<bool> {
        let args: Vec<String> = line.trim()
            .split_whitespace()
            .map(|s| s.to_string())
            .collect();

        if args.is_empty() {
            return Ok(true);
        }
        
        let cmd = args[0].to_lowercase();
        
        if cmd == "exit" || cmd == "quit" {
            let mut iqos = self.iqos.lock().await;
            let _ = iqos.disconnect().await?;
            return Ok(false);
        }
        
        if let Some(handler) = self.commands.lock().await.get(&cmd) {
            handler(self, args).await?;
        } else {
            println!("Unknown command: {}. Use 'help' to display available commands.", cmd);
        }
        
        Ok(true)
    }
    
    pub async fn run(&mut self) -> Result<()> {
        println!("IQOS Command Console v0.1.0");
        println!("Type 'help' to display available commands, 'exit' to quit");
        
        let config = Config::builder().build();
        let mut rl = Editor::<IqosHelper, rustyline::history::DefaultHistory>::with_config(config)?;
        let helper = IqosHelper::new();
        rl.set_helper(Some(helper));
        
        if rl.load_history("history.txt").is_err() {
            println!("No history file found");
        }
        
        self.running = true;
        
        while self.running {
            match rl.readline("iqos> ") {
                Ok(line_str) => {
                    let _ = rl.add_history_entry(&line_str);
                    
                    match self.execute(&line_str).await {
                        Ok(continue_running) => {
                            if !continue_running {
                                break;
                            }
                        },
                        Err(e) => {
                            println!("Command execution error: {}", e);
                        }
                    }
                },
                Err(ReadlineError::Interrupted) => {
                    println!("Ctrl-C");
                    break;
                },
                Err(ReadlineError::Eof) => {
                    println!("Ctrl-D");
                    break;
                },
                Err(err) => {
                    println!("Error: {:?}", err);
                    break;
                }
            }
        }
        
        rl.save_history("history.txt")?;
        
        Ok(())
    }
}

// For backward compatibility
pub type CommandSystem = IQOSConsole;

// Function for backward compatibility
pub async fn run_console(iqos: IqosBle) -> Result<()> {
    let mut console = IQOSConsole::new(iqos);
    console.run().await
}
