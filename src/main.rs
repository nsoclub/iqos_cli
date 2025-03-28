use btleplug::api::{
    bleuuid::uuid_from_u16, Central, CentralEvent, Manager as _, Peripheral as _, ScanFilter,
    WriteType,
};
use btleplug::platform::{Adapter, Manager, Peripheral};
use futures::stream::StreamExt;
use futures::future::{self, TryFutureExt};
use std::error::Error;
use std::time::Duration;
use uuid::Uuid;

const LIGHT_CHARACTERISTIC_UUID: Uuid = uuid_from_u16(0xFFE9);

async fn get_central(manager: &Manager) -> Adapter {
    let adapters = manager.adapters().await.unwrap();
    adapters.into_iter().nth(0).unwrap()
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
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
                    let iqos = peripheral;
                    println!("Found IQOS: {name} ({addr})");
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

    // // dance party
    // let mut rng = thread_rng();
    // for _ in 0..20 {
    //     let color_cmd = vec![0x56, rng.gen(), rng.gen(), rng.gen(), 0x00, 0xF0, 0xAA];
    //     light
    //         .write(&cmd_char, &color_cmd, WriteType::WithoutResponse)
    //         .await?;
    //     time::sleep(Duration::from_millis(200)).await;
    // }
    Ok(())
}