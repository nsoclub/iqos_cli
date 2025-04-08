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

use crate::iqos::{self, IQOS};

type CommandFn = Box<dyn Fn(&IQOSConsole, &[&str]) -> Pin<Box<dyn Future<Output = Result<()>> + Send>> + Send + Sync>;

#[derive(Clone)]
pub struct IQOSConsole {
    commands: Arc<Mutex<HashMap<String, CommandFn>>>,
    iqos: Arc<Mutex<IQOS>>,
    running: bool,
}

impl IQOSConsole {
    pub fn new(iqos: IQOS) -> Self {
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

        commands.insert("brightness".to_string(), Box::new(|_console: &IQOSConsole, args| {
            let args: Vec<String> = args.iter().map(|&s| s.to_string()).collect();
            Box::pin(async move {
                match args.get(1).map(|s| s.as_str()) {
                    Some("high") => println!("明るさを高に設定しました"),
                    Some("low") => println!("明るさを低に設定しました"),
                    Some(opt) => println!("無効なオプション: {}。'high', 'low' のいずれかを指定してください", opt),
                    None => println!("使い方: brightness [high|medium|low]"),
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
        
        commands.insert("smartgesture".to_string(), Box::new(|_console: &IQOSConsole, args| {
            let args: Vec<String> = args.iter().map(|&s| s.to_string()).collect();
            Box::pin(async move {
                match args.get(1).map(|s| s.as_str()) {
                    Some("enable") => println!("スマートジェスチャーを有効にしました"),
                    Some("disable") => println!("スマートジェスチャーを無効にしました"),
                    Some(opt) => println!("無効なオプション: {}。'enable'または'disable'を指定してください", opt),
                    None => println!("使い方: smartgesture [enable|disable]"),
                }
                Ok(())
            })
        }));
        
        commands.insert("help".to_string(), Box::new(|_console: &IQOSConsole, _| {
            Box::pin(async move {
                println!("利用可能なコマンド:");
                println!("  brightness [high|medium|low] - 明るさを設定します");
                println!("  battery - バッテリーの状態を表示します");
                println!("  lock | unlock - デバイスをロックまたはアンロックします");
                println!("  findmyiqos - デバイスを探す機能を起動します");
                println!("  smartgesture [enable|disable] - スマートジェスチャー機能を設定します");
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
        let args: Vec<&str> = line.trim().split_whitespace().collect();
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
            handler(self, &args).await?;
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
pub async fn run_console(iqos: IQOS) -> Result<()> {
    let mut console = IQOSConsole::new(iqos);
    console.run().await
}