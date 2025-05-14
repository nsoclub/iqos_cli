use rustyline::error::ReadlineError;
use rustyline::{Config, DefaultEditor, Editor};
use anyhow::Result;
use tokio::sync::Mutex;

use std::collections::HashMap;
use std::pin::Pin;
use std::sync::Arc;
use std::future::Future;

mod iqoshelper;
use iqoshelper::IqosHelper;

use crate::iqos::vibration::IlumaVibrationBehavior;
use crate::iqos::{IqosBle, BrightnessLevel, IqosIluma, Iqos, VibrationSettings, vibration::VibrationBehavior};

// クロージャーの型定義を修正
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
                            println!("明るさを{}に設定しました", level);
                        } else {
                            println!("このデバイスはILUMAモデルではありません");
                        }
                    },
                    Some(Err(e)) => println!("{}", e),
                    None => iqos.load_brightness().await.map(|level| {
                        println!("{}", level);
                    }).unwrap_or_else(|_| {
                        println!("このデバイスはILUMAモデルではありません");
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

        commands.insert("flexpuff".to_string(), Box::new(|console: &IQOSConsole, args| {
            let iqos = console.iqos.clone();
            let args_cloned = args.clone();
            Box::pin(async move {
                let iqos = iqos.lock().await;
                match args_cloned.get(1).map(|s| s.as_str()) {
                    Some("enable") => {
                        let result = iqos.update_flexpuff(true).await;
                        match result {
                            Ok(_) => println!("Flexpuff enabled"),
                            Err(e) => println!("エラー: {}", e),
                        }
                    },
                    Some("disable") => {
                        let result = iqos.update_flexpuff(false).await;
                        match result {
                            Ok(_) => println!("Flexpuff disabled"),
                            Err(e) => println!("エラー: {}", e),
                        }
                    },
                    Some(opt) => println!("無効なオプション: {}。'enable'または'disable'を指定してください", opt),
                    None => println!("使い方: flexpuff [enable|disable]"),
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
                            // Turn the future into a proper Send future
                            let result = iluma.update_smartgesture(true).await;
                            match result {
                                Ok(_) => println!("スマートジェスチャーを有効にしました"),
                                Err(e) => println!("エラー: {}", e),
                            }
                        },
                        Some("disable") => {
                            // Turn the future into a proper Send future
                            let result = iluma.update_smartgesture(false).await;
                            match result {
                                Ok(_) => println!("スマートジェスチャーを無効にしました"),
                                Err(e) => println!("エラー: {}", e),
                            }
                        },
                        Some(opt) => println!("無効なオプション: {}。'enable'または'disable'を指定してください", opt),
                        None => println!("使い方: smartgesture [enable|disable]"),
                    }
                } else {
                    println!("このデバイスはILUMAモデルではありません");
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
                                Err(e) => println!("エラー: {}", e),
                            }
                        },
                        s if s == "off" || s == "disable" => {
                            let result = iqos.update_autostart(false).await;
                            match result {
                                Ok(_) => println!("Autostart disabled"),
                                Err(e) => println!("エラー: {}", e),
                            }
                        },
                        _ => println!("使い方: autostart [on|off]"),
                    }
                } else {
                    println!("使い方: autostart [on|off]");
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
                        iluma.update_vibration_settings(settings).await?;
                        println!("バイブレーション設定を更新しました");
                    } else {
                        let settings = VibrationSettings::from_args(param_args)?;
                        iqos.update_vibration_settings(settings).await?;
                        println!("バイブレーション設定を更新しました");
                    }
                } else if args_cloned.len() == 1 {
                    if let Some(iluma) = iqos.as_iluma() {
                        match iluma.load_iluma_vibration_settings().await {
                            Ok(settings) => {
                                println!("is iluma : {:?}", settings);
                                println!("{}", settings);
                            },
                            Err(e) => println!("エラー: {}", e),
                        }
                    } else {
                        match iqos.load_vibration_settings().await {
                            Ok(settings) => println!("{}", settings),
                            Err(e) => println!("エラー: {}", e),
                        }
                    }
                } else {
                    println!("使い方: vibration [charge|heating|starting|terminated|puffend] [on|off] ...");
                    println!("例: vibration charge on heating on puffend on");
                    println!("注: chargeオプションはILUMAモデルのみ対応");
                }
                Ok(())
            })
        }));
        
        commands.insert("help".to_string(), Box::new(|console: &IQOSConsole, _| {
            let iqos = console.iqos.clone();
            Box::pin(async move {
                let iqos = iqos.lock().await;
                let is_iluma = iqos.is_iluma();
                
                println!("利用可能なコマンド:");
                
                // 共通コマンド
                println!("  battery - バッテリーの状態を表示します");
                println!("  lock | unlock - デバイスをロックまたはアンロックします");
                println!("  findmyiqos - デバイスを探す機能を起動します");
                println!("  autostart [on|off] - オートスタート機能を設定します");
                
                // ILUMAモデル固有のコマンド
                if is_iluma {
                    println!("\nILUMAモデル固有のコマンド:");
                    println!("  brightness [high|low] - 明るさを設定します");
                    println!("  smartgesture [enable|disable] - スマートジェスチャー機能を設定します");
                    println!("  flexpuff [enable|disable] - フレックスパフ機能を設定します");
                    println!("  vibration [option on|off]... - バイブレーション設定を構成します");
                    println!("    例: vibration charge on heating on puffend on");
                    println!("    オプション: charge, heating, starting, terminated, puffend");
                }
                
                println!("\nその他のコマンド:");
                println!("  info - デバイスのステータスを表示します");
                println!("  help - このヘルプメッセージを表示します");
                println!("  quit | exit - プログラムを終了します");
                
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
            println!("不明なコマンド: {}。'help'で利用可能なコマンドを表示します。", cmd);
        }
        
        Ok(true)
    }
    
    pub async fn run(&mut self) -> Result<()> {
        println!("IQOS コマンドコンソール v0.1.0");
        println!("'help'で利用可能なコマンドを表示、'exit'で終了します");
        
        let config = Config::builder().build();
        let mut rl = Editor::<IqosHelper, rustyline::history::DefaultHistory>::with_config(config)?;
        let helper = IqosHelper::new();
        rl.set_helper(Some(helper));
        
        if rl.load_history("history.txt").is_err() {
            println!("履歴ファイルが見つかりません");
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
                            println!("コマンド実行エラー: {}", e);
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
                    println!("エラー: {:?}", err);
                    break;
                }
            }
        }
        
        rl.save_history("history.txt")?;
        
        Ok(())
    }
}

// 互換性のために古い型名を提供
pub type CommandSystem = IQOSConsole;

// 互換性のための関数
pub async fn run_console(iqos: IqosBle) -> Result<()> {
    let mut console = IQOSConsole::new(iqos);
    console.run().await
}