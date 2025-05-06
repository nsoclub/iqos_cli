#[cfg(test)]
mod tests {
    use btleplug::api::{Central, Manager as _, Peripheral as _, ScanFilter, CentralEvent};
    use btleplug::platform::Manager;
    use futures::stream::StreamExt;
    use crate::iqos::builder::IQOSBuilder;
    use crate::iqos::error::{Result, IQOSError};

    async fn discover_and_print_services(iqos: &mut IQOSBuilder) -> Result<()> {
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
                        
                        // Device Information Serviceの場合、characteristic UUIDを判別
                        if service.uuid.to_string() == "0000180a-0000-1000-8000-00805f9b34fb" {
                            let uuid_string = characteristic.uuid.to_string();
                            let uuid_short = uuid_string.split('-').next().unwrap_or("");
                            match uuid_short {
                                "00002a24" => println!("        標準特性: Model Number String"),
                                "00002a25" => println!("        標準特性: Serial Number String"),
                                "00002a28" => println!("        標準特性: Software Revision String"),
                                "00002a29" => println!("        標準特性: Manufacturer Name String"),
                                _ => println!("        標準特性: 未定義"),
                            }
                        }
                        
                        // 読み取り可能な場合は値を読み取って表示
                        if characteristic.properties.contains(btleplug::api::CharPropFlags::READ) {
                            print!("        読み取り中...");
                            if let Ok(p) = iqos.peripheral() {
                                match p.read(&characteristic).await {
                                    Ok(data) => {
                                        if let Ok(text) = String::from_utf8(data.clone()) {
                                            if text.chars().all(|c| c.is_ascii_graphic() || c.is_ascii_whitespace()) {
                                                println!("値 (文字列): {}", text);
                                            } else {
                                                println!("値 (ASCII): {}", data.iter()
                                                    .map(|&b| b.to_ascii_lowercase() as char)
                                                    .collect::<String>());
                                            }
                                        } else {
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
        Ok(())
    }

    #[tokio::test]
    async fn test_discover_services() -> Result<()> {
        let manager = Manager::new().await.map_err(IQOSError::from)?;
        let adapters = manager.adapters().await.map_err(IQOSError::from)?;
        let central = adapters.into_iter().nth(0).ok_or("No Bluetooth adapter found")?;

        let mut events = central.events().await.map_err(IQOSError::from)?;
        central.start_scan(ScanFilter::default()).await.map_err(IQOSError::from)?;

        while let Some(event) = events.next().await {
            if let CentralEvent::DeviceDiscovered(addr) = event {
                let peripheral = central.peripheral(&addr).await.map_err(IQOSError::from)?;
                let properties = peripheral.properties().await.map_err(IQOSError::from)?;
                if let Some(name) = properties.and_then(|p| p.local_name) {
                    if name.contains("IQOS") {
                        let mut iqos = IQOSBuilder::new(peripheral);
                        iqos.connect().await?;
                        discover_and_print_services(&mut iqos).await?;
                        central.stop_scan().await.map_err(IQOSError::from)?;
                        break;
                    }
                }
            }
        }
        Ok(())
    }
}

// チェックサムテスト用のモジュールは別ファイルで定義済み
#[cfg(test)]
mod iqos_checksum_tests;