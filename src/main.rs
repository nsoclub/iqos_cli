use std::io::{stdin,stdout,Write};
use btleplug::api::{
    bleuuid::uuid_from_u16, Central, CentralEvent, Manager as _, Peripheral as _, ScanFilter,
    WriteType,
};
use btleplug::platform::{Adapter, Manager, Peripheral};
use futures::stream::StreamExt;
use futures::future::{self, TryFutureExt};
use std::error::Error;
use uuid::Uuid;

mod iqos;
mod console;

use console::run_console;

async fn get_central(manager: &Manager) -> Adapter {
    let adapters = manager.adapters().await.unwrap();
    adapters.into_iter().nth(0).unwrap()
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // let mut iqos = iqos::IQOS::new();
    let mut iqos: iqos::IQOSBuilder;
    let manager = Manager::new().await.unwrap();

    // get the first bluetooth adapter
    let central = get_central(&manager).await;

    let central_state = central.adapter_state().await.unwrap();
    println!("CentralState: {:?}", central_state);

    let mut events = central.events().await?;
    // start scanning for devices
    central.start_scan(ScanFilter::default()).await?;

    while let Some(event) = events.next().await {
        match event {
            CentralEvent::DeviceDiscovered(addr) => {
                let peripheral = central.peripheral(&addr).await?;
                let properties = peripheral.properties().await?;
                let name = properties
                    .and_then(|p| p.local_name)
                    .map(|local_name| local_name.to_string())
                    .unwrap_or_default();

                println!("Device Discovered: {name} ({addr})");
                if name.contains("IQOS") {
                    iqos = iqos::IQOSBuilder::new();
                    iqos = iqos.with_peripheral(peripheral).await?;
                    let is_connected = iqos.is_connected().await?;

                    println!("Found IQOS: {name} ({addr})");
                    print!("Connect to {name} ({addr})? [y/N]: ");

                    let _  = std::io::stdout().flush();
                    let mut input = String::new();
                    std::io::stdin().read_line(&mut input)?;

                    if input.trim().to_lowercase() == "y" {
                        println!("Connecting to IQOS device...");
                        iqos.connect().await?;
                        println!("Connected! Discovering services...");
                        let services = iqos.discover_services().await?;
                        
                        println!("発見されたサービス:");
                        if services.is_empty() {
                            println!("  サービスが見つかりませんでした");
                        } else {
                            for (i, service) in services.iter().enumerate() {
                                println!("  サービス #{}: {}", i + 1, service.uuid);
                                println!("    プライマリ: {}", service.primary);
                                
                                if !service.characteristics.is_empty() {
                                    println!("    特性:");
                                    for (j, characteristic) in service.characteristics.iter().enumerate() {
                                        println!("      特性 #{}.{}: {}", i + 1, j + 1, characteristic.uuid);
                                        println!("        プロパティ: {:?}", characteristic.properties);
                                        
                                        // 標準特性の場合は名前も表示
                                        match characteristic.uuid.to_string().as_str() {
                                            "00002a24-0000-1000-8000-00805f9b34fb" => println!("        名前: モデル番号"),
                                            "00002a25-0000-1000-8000-00805f9b34fb" => println!("        名前: シリアル番号"),
                                            "00002a26-0000-1000-8000-00805f9b34fb" => println!("        名前: ファームウェアリビジョン"),
                                            "00002a27-0000-1000-8000-00805f9b34fb" => println!("        名前: ハードウェアリビジョン"),
                                            "00002a29-0000-1000-8000-00805f9b34fb" => println!("        名前: 製造者名"),
                                            "00002a19-0000-1000-8000-00805f9b34fb" => println!("        名前: バッテリーレベル"),
                                            _ => {}
                                        }
                                        
                                        // 読み取り可能な場合は値を読み取って表示
                                        if characteristic.properties.contains(btleplug::api::CharPropFlags::READ) {
                                            print!("        読み取り中...");
                                            // iqosからperipheralを取得して読み取り
                                            if let Ok(p) = iqos.peripheral() {
                                                match p.read(&characteristic).await {
                                                    Ok(data) => {
                                                        if let Ok(text) = String::from_utf8(data.clone()) {
                                                            if text.chars().all(|c| c.is_ascii_graphic() || c.is_ascii_whitespace()) {
                                                                println!("値 (文字列): {}", text);
                                                            } else {
                                                                println!("値 (バイト): {:?}", data);
                                                            }
                                                        } else {
                                                            // バイナリデータの場合は16進数で表示
                                                            println!("値 (16進数): {}", data.iter()
                                                                .map(|b| format!("{:02X}", b))
                                                                .collect::<Vec<_>>()
                                                                .join(" "));
                                                        }
                                                    },
                                                    Err(e) => println!("読み取りエラー: {}", e)
                                                }
                                            } else {
                                                println!("peripheralの取得に失敗");
                                            }
                                        }
                                    }
                                } else {
                                    println!("    特性: なし");
                                }
                                println!();
                            }
                        }

                        let iqos = iqos.build()?;
                        // コンソールを起動して対話的な操作を開始
                        run_console(iqos)?;
                        
                    }
                    
                    // スキャンを停止して終了
                    central.stop_scan().await?;
                    break;
                }

            }
            CentralEvent::StateUpdate(state) => {
                println!("State Update: {:?}", state);
            }
            CentralEvent::DeviceConnected(id) => {
                println!("Device Connected: {id}");
            }
            CentralEvent::DeviceDisconnected(id) => {
                println!("Device Disconnected: {id}");
            }
            _ => {}
        }
    }
    Ok(())
}