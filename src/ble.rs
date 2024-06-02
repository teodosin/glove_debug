// Code for the bluetooth client implementation

use bevy::app::{App, Plugin, Startup};
use bevy::ecs::system::ResMut;
use btleplug::api::{Central, Characteristic, Manager as _, Peripheral, ScanFilter};
use btleplug::platform::Manager;
use std::time::Duration;
use tokio::time;

use crate::asyncs::{TaskContext, TokioTasksPlugin, TokioTasksRuntime};
use crate::RukaInput;

pub struct BLEPlugin;

impl Plugin for BLEPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_plugins(TokioTasksPlugin::default())
            .add_systems(Startup, connect)
        ;
    }
}

fn connect(runtime: ResMut<TokioTasksRuntime>) {
    // do the bluetooth connection thingy
    runtime.spawn_background_task(try_connect);

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

                ctx.run_on_main_thread(move |main_ctx| {
                    let mut ruka = main_ctx.world.get_resource_mut::<RukaInput>().unwrap();
                    ruka.set_init(true);
                }).await;

                while is_connected {
                    // if !peripheral.is_connected().await.unwrap() {
                    //     println!("Disconnected from peripheral {:?}...", &local_name);
                    //     break;
                    // }
                    let mut value: u16 = 0;

                    for service in peripheral.services() {
                        // println!(
                        //     "Service UUID {}, primary: {}",
                        //     service.uuid, service.primary
                        // );
                        for characteristic in service.characteristics {
                            if characteristic.uuid.to_string() != "00002af9-0000-1000-8000-00805f9b34fb" {
                                continue;
                            }
                            
                            // println!("Could find");
                            // for descriptor in &characteristic.descriptors {
                            //     println!("    Descriptor UUID: {}", descriptor);
                            // }
                            
                            println!("Trying to read {:?}", characteristic.uuid.to_string());
                            let read_result = peripheral.read(&characteristic).await;
                            match read_result {
                                Ok(data) => {
                                    if data.len() == 2 {
                                        let high_byte = data[0];
                                        let low_byte = data[1];
                            
                                        // Combine the high and low bytes to get the original 16-bit value
                                        value = ((high_byte as u16) << 6) | (low_byte as u16);
                                        println!("Read bytes: {:?}", value);
                                    }
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

                    if is_connected {
                        ctx.run_on_main_thread(move |main_ctx| {
                            let mut ruka = main_ctx.world.get_resource_mut::<RukaInput>().unwrap();
                            ruka.update_finger(0, value);
                        }).await;
                    }
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