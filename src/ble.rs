// Code for the bluetooth client implementation

use bevy::app::{App, Plugin, Startup};
use bevy::ecs::system::ResMut;
use btleplug::api::{Central, Characteristic, Manager as _, Peripheral, ScanFilter};
use btleplug::platform::Manager;
use std::time::Duration;
use tokio::time;

use crate::asyncs::{TaskContext, TokioTasksPlugin, TokioTasksRuntime};
use crate::ruka::RukaInput;

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
                    let mut flexvalues: [u16; 5] = [0; 5];
                    let mut imuvalues: [f32; 6] = [0.0; 6];

                    for service in peripheral.services() {
                        // println!(
                        //     "Service UUID {}, primary: {}",
                        //     service.uuid, service.primary
                        // );
                        for characteristic in service.characteristics {
                            if characteristic.uuid.to_string() == "00002af9-0000-1000-8000-00805f9b34fb" {
                                // println!("Trying to read {:?}", characteristic.uuid.to_string());
                                let read_result = peripheral.read(&characteristic).await;
                                match read_result {
                                    Ok(data) => {
                                        if data.len() == 10 {
                                            for i in 0..5 {
                                                let high_byte = data[2*i];
                                                let low_byte = data[2*i+1];
    
                                                flexvalues[i] = ((high_byte as u16) << 8) | (low_byte as u16);
                                            }
    
                                        }
                                    }
                                    Err(err) => {
                                        eprintln!("Error reading characteristic: {}", err);
                                    }
                                }
                                
                            } else if characteristic.uuid.to_string() == "00002713-0000-1000-8000-00805f9b34fb" {
                                // println!("Trying to read {:?}", characteristic.uuid.to_string());
                                let read_result = peripheral.read(&characteristic).await;
                                match read_result {
                                    Ok(data) => {
                                        if data.len() == 12 {
                                            for i in 0..6 {
                                                let high_byte = data[i * 2] as i16;
                                                let low_byte = data[i * 2 + 1] as i16;
                                                let int_value = (high_byte << 8) | (low_byte & 0xFF );
                                                let float_value = int_value as f32 / 100.0;
                                                imuvalues[i] = float_value;
                                            }
                                            
                                        }
                                    }
                                    Err(err) => {
                                        eprintln!("Error reading characteristic: {}", err);
                                    }
                                }
                            } else {
                                continue;
                            }
                            println!("Read flex bytes: {:?}", flexvalues);
                            println!("Read imu bytes: {:?}", imuvalues);
                            
                        }
                    }

                    let delay = Duration::from_millis(1000 / 60);
                    // tokio::time::sleep(delay).await;

                    if is_connected {
                        ctx.run_on_main_thread(move |main_ctx| {
                            let mut ruka = main_ctx.world.get_resource_mut::<RukaInput>().unwrap();
                            ruka.update_fingers(flexvalues);
                            ruka.update_imu(imuvalues);
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