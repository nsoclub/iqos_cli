use rustyline::error::ReadlineError;
use rustyline::{Editor, Config};
use anyhow::{Result, Context};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

mod iqoshelper;
use iqoshelper::IqosHelper;

use crate::iqos::IQOS;

/// コマンドハンドラの型は関数ポインタではなくクロージャとして定義
type CommandFn = Box<dyn Fn(&IQOSConsole, &[&str]) -> Result<()>>;

/// IQOSコンソールの主要構造体
pub struct IQOSConsole {
    commands: HashMap<String, CommandFn>,
    iqos: Arc<Mutex<IQOS>>,
    running: bool,
}

impl IQOSConsole {
    /// 新しいIQOSコンソールを作成
    pub fn new(iqos: IQOS) -> Self {
        let mut console = Self {
            commands: HashMap::new(),
            iqos: Arc::new(Mutex::new(iqos)),
            running: false,
        };
        
        // コマンドを登録
        console.register_commands();
        console
    }
    
    /// コマンドを登録するメソッド
    fn register_commands(&mut self) {
        // 各コマンドをクロージャとして登録
        let commands = &mut self.commands;
        
        commands.insert("brightness".to_string(), Box::new(|console, args| {
            console.handle_brightness(args)
        }));
        
        commands.insert("findmyiqos".to_string(), Box::new(|console, args| {
            console.handle_find_my_iqos(args)
        }));
        
        commands.insert("smartgesture".to_string(), Box::new(|console, args| {
            console.handle_smart_gesture(args)
        }));
        
        commands.insert("help".to_string(), Box::new(|console, args| {
            console.handle_help(args)
        }));
        
        commands.insert("info".to_string(), Box::new(|console, args| {
            console.handle_info(args)
        }));
    }
    
    /// コマンドを実行
    fn execute(&self, line: &str) -> Result<bool> {
        let args: Vec<&str> = line.trim().split_whitespace().collect();
        if args.is_empty() {
            return Ok(true); // 空行は無視
        }
        
        let cmd = args[0].to_lowercase();
        
        if cmd == "exit" || cmd == "quit" {
            if let Ok(mut iqos) = self.iqos.lock() {
                let _ = iqos.disconnect();
            }
            return Ok(false); // 終了
        }
        
        if let Some(handler) = self.commands.get(&cmd) {
            handler(self, &args)?;
        } else {
            println!("不明なコマンド: {}。'help'で利用可能なコマンドを表示します。", cmd);
        }
        
        Ok(true)
    }
    
    /// コンソールを実行
    pub fn run(&mut self) -> Result<()> {
        println!("IQOS コマンドコンソール v0.1.0");
        println!("'help'で利用可能なコマンドを表示、'exit'で終了します");
        
        let config = Config::builder().build();
        let mut rl = Editor::<IqosHelper, rustyline::history::DefaultHistory>::with_config(config)?;
        let helper = IqosHelper::new();
        rl.set_helper(Some(helper));
        
        // 履歴ファイルのロード
        if rl.load_history("history.txt").is_err() {
            println!("履歴ファイルが見つかりません");
        }
        
        self.running = true;
        
        // メインループ
        while self.running {
            match rl.readline("iqos> ") {
                Ok(line_str) => {
                    let _ = rl.add_history_entry(&line_str);
                    
                    match self.execute(&line_str) {
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
        
        // 履歴の保存
        rl.save_history("history.txt")?;
        
        Ok(())
    }
    
    // ========== コマンドハンドラ ==========
    
    /// 明るさ設定コマンド
    fn handle_brightness(&self, args: &[&str]) -> Result<()> {
        if args.len() < 2 {
            println!("使い方: brightness [high|medium|low]");
            return Ok(());
        }
        
        match args[1].to_lowercase().as_str() {
            "high" => {
                println!("明るさを高に設定しました");
                // 実際のIQOS操作コードは以下に実装
                // let mut iqos = self.iqos.lock().unwrap();
                // iqos.set_brightness("high")?;
            },
            "medium" => {
                println!("明るさを中に設定しました");
                // 実際のIQOS操作コード
            },
            "low" => {
                println!("明るさを低に設定しました");
                // 実際のIQOS操作コード
            },
            _ => println!("無効なオプション: {}。'high', 'medium', 'low' のいずれかを指定してください", args[1]),
        }
        
        Ok(())
    }
    
    /// デバイス検索コマンド
    fn handle_find_my_iqos(&self, _args: &[&str]) -> Result<()> {
        println!("IQOSを探しています...");
        // 実際のIQOS操作コード
        // let mut iqos = self.iqos.lock().unwrap();
        // iqos.find_my_device()?;
        println!("Find my IQOS機能が起動しました");
        Ok(())
    }
    
    /// スマートジェスチャー設定コマンド
    fn handle_smart_gesture(&self, args: &[&str]) -> Result<()> {
        if args.len() < 2 {
            println!("使い方: smartgesture [enable|disable]");
            return Ok(());
        }
        
        match args[1].to_lowercase().as_str() {
            "enable" => {
                println!("スマートジェスチャーを有効にしました");
                // 実際のIQOS操作コード
                // let mut iqos = self.iqos.lock().unwrap();
                // iqos.set_smart_gesture(true)?;
            },
            "disable" => {
                println!("スマートジェスチャーを無効にしました");
                // 実際のIQOS操作コード
                // let mut iqos = self.iqos.lock().unwrap();
                // iqos.set_smart_gesture(false)?;
            },
            _ => println!("無効なオプション: {}。'enable'または'disable'を指定してください", args[1]),
        }
        
        Ok(())
    }
    
    /// ステータス表示コマンド
    fn handle_info(&self, _args: &[&str]) -> Result<()> {
        // 実際のIQOS操作コード
        // let iqos = self.iqos.lock().unwrap();
        // let status = iqos.get_status()?;
        // println!("バッテリー残量: {}%", status.battery_level);
        // println!("接続状態: {}", if status.is_connected { "接続中" } else { "未接続" });
        
        // 仮のステータス表示
        if let Ok(iqos) = self.iqos.lock() {
            println!("\n{}\n", iqos)
        }
        Ok(())
    }
    
    /// ヘルプコマンド
    fn handle_help(&self, _args: &[&str]) -> Result<()> {
        println!("利用可能なコマンド:");
        println!("  brightness [high|medium|low] - 明るさを設定します");
        println!("  findmyiqos - デバイスを探す機能を起動します");
        println!("  smartgesture [enable|disable] - スマートジェスチャー機能を設定します");
        println!("  info - デバイスのステータスを表示します");
        println!("  help - このヘルプメッセージを表示します");
        println!("  exit - プログラムを終了します");
        Ok(())
    }
    
    /// 終了コマンド
    fn handle_exit(&self, _args: &[&str]) -> Result<()> {
        println!("終了します");
        Ok(())
    }
}

// 互換性のために古い型名を提供
pub type CommandSystem = IQOSConsole;

// 互換性のための関数
pub fn run_console(iqos: IQOS) -> Result<()> {
    let mut console = IQOSConsole::new(iqos);
    console.run()
}