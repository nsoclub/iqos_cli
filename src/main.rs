use std::io::Write;
use btleplug::api::{
    Central, CentralEvent, Manager as _, Peripheral as _, ScanFilter,
};
use btleplug::platform::{Adapter, Manager};
use futures::stream::StreamExt;
use std::error::Error;

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
                    iqos = iqos::IQOSBuilder::new(peripheral);

                    println!("Found IQOS: {name} ({addr})");
                    print!("Connect to {name} ({addr})? [y/N]: ");

                    let _  = std::io::stdout().flush();
                    let mut input = String::new();
                    std::io::stdin().read_line(&mut input)?;

                    if input.trim().to_lowercase() == "y" {
                        println!("Connecting to IQOS device...");
                        iqos.connect().await?;
                        println!("Connected!");
                        let _services = iqos.discover_services().await?;
                        iqos.initialize().await?;

                        let iqos = iqos.build().await?;
                        // コンソールを起動して対話的な操作を開始
                        run_console(iqos).await?;
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