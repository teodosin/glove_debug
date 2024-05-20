mod asyncs;
mod ble;
mod particles;

use asyncs::{TaskContext, TokioTasksPlugin, TokioTasksRuntime};
use bevy::prelude::*;
use btleplug::api::{Central, Characteristic, Manager as _, Peripheral, ScanFilter};
use btleplug::platform::Manager;
use particles::ParticlePlugin;
use std::time::Duration;
use tokio::time;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(TokioTasksPlugin::default())
        .add_plugins(ParticlePlugin)
        
        .insert_resource(RukaInput { latest: 0 })

        .add_systems(Startup, connect)
        .add_systems(Update, listen)
    .run();
}

#[derive(Resource, Deref)]
struct RukaInput {
    latest: u16
}

fn connect(runtime: ResMut<TokioTasksRuntime>, mut commands: Commands) {
    // do the bluetooth connection thingy
    runtime.spawn_background_task(try_connect);

}

fn listen() {
    // nothing right now
}

async fn try_connect(mut ctx: TaskContext) {
    let manager = Manager::new().await.expect("Failed to create BLE manager");
    let adapter_list = manager.adapters().await.expect("Failed to get adapter list");
    if adapter_list.is_empty() {
        eprintln!("No Bluetooth adapters found");
        return;
    }

    for adapter in adapter_list.iter() {
        println!("Starting scan on {}...", adapter.adapter_info().await.expect("Failed to get adapter info"));
        
        
        adapter
            .start_scan(ScanFilter::default())
            .await
            .expect("Can't scan BLE adapter for connected devices...");


        time::sleep(Duration::from_secs(20)).await;
        let peripherals = adapter.peripherals().await.expect("Failed to get peripherals");
        if peripherals.is_empty() {
            eprintln!("->>> BLE peripheral devices were not found, sorry. Exiting...");
        } else {

            let target_name = "Ruka";

            // All peripheral devices in range
            for peripheral in peripherals.iter() {
                let properties = peripheral.properties().await.expect("Failed to get peripheral properties");
                let is_connected = peripheral.is_connected().await.expect("Failed to get connection status");
                let local_name = properties
                    .unwrap()
                    .local_name
                    .unwrap_or(String::from("(peripheral name unknown)"));

                if local_name != target_name {
                    continue;
                }

                println!(
                    "Peripheral {:?} is connected: {:?}",
                    local_name, is_connected
                );
                if !is_connected {
                    println!("Connecting to peripheral {:?}...", &local_name);
                    if let Err(err) = peripheral.connect().await {
                        eprintln!("Error connecting to peripheral, skipping: {}", err);
                        continue;
                    }
                }
                let is_connected = peripheral.is_connected().await.expect("Failed to get connection status");
                
                println!(
                    "Now connected ({:?}) to peripheral {:?}...",
                    is_connected, &local_name
                );

                println!("Discover peripheral {:?} services...", &local_name);
                peripheral.discover_services().await.expect("Failed to discover services");

                while is_connected {
                    // if !peripheral.is_connected().await.unwrap() {
                    //     println!("Disconnected from peripheral {:?}...", &local_name);
                    //     break;
                    // }

                    for service in peripheral.services() {
                        // println!(
                        //     "Service UUID {}, primary: {}",
                        //     service.uuid, service.primary
                        // );
                        for characteristic in service.characteristics {
                            // if characteristic.uuid.to_string() != "1efca1a0-3360-4fb4-9070-b1e9ef5079a9" {
                            //     continue;
                            // }
                            
                            // println!("Could find");
                            // for descriptor in &characteristic.descriptors {
                            //     println!("    Descriptor UUID: {}", descriptor);
                            // }
                            
                            println!("Trying to read {:?}", characteristic.uuid.to_string());
                            let read_result = peripheral.read(&characteristic).await;
                            match read_result {
                                Ok(data) => {
                                    // let value = data;
                                    let value = unsafe { std::str::from_utf8_unchecked(&data)};
                                    // let value = u16::from_le_bytes([data[0], data[1]]);
                                    println!("Read bytes: {:?}", value);
                                    println!("---------------------------------------");
                                }
                                Err(err) => {
                                    eprintln!("Error reading characteristic: {}", err);
                                }
                            }
                        }
                    }

                    let delay = Duration::from_millis(1000 / 60);
                    // tokio::time::sleep(delay).await;

                    // if is_connected {
                    //     ctx.run_on_main_thread(move |mut main_ctx| {
                    //         main_ctx.world.insert_resource(RukaInput { latest: 0 });
                    //     });
                    // }
                }
                // if is_connected {
                //     // println!("Disconnecting from peripheral {:?}...", &local_name);
                //     // peripheral
                //     //     .disconnect()
                //     //     .await
                //     //     .expect("Error disconnecting from BLE peripheral");
                // }
            }
        }
    }
}