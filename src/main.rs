mod asyncs;
mod ble;

use asyncs::{TaskContext, TokioTasksPlugin, TokioTasksRuntime};
use bevy::prelude::*;
use btleplug::api::{Central, Manager as _, Peripheral, ScanFilter};
use btleplug::platform::Manager;
use std::time::Duration;
use tokio::time;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(TokioTasksPlugin::default())
        .add_systems(Startup, connect)
        .add_systems(Update, listen)
    .run();
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
        time::sleep(Duration::from_secs(10)).await;
        let peripherals = adapter.peripherals().await.expect("Failed to get peripherals");
        if peripherals.is_empty() {
            eprintln!("->>> BLE peripheral devices were not found, sorry. Exiting...");
        } else {
            // All peripheral devices in range
            for peripheral in peripherals.iter() {
                let properties = peripheral.properties().await.expect("Failed to get peripheral properties");
                let is_connected = peripheral.is_connected().await.expect("Failed to get connection status");
                let local_name = properties
                    .unwrap()
                    .local_name
                    .unwrap_or(String::from("(peripheral name unknown)"));
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
                peripheral.discover_services().await.expect("Failed to discover services");
                println!("Discover peripheral {:?} services...", &local_name);
                for service in peripheral.services() {
                    println!(
                        "Service UUID {}, primary: {}",
                        service.uuid, service.primary
                    );
                    for characteristic in service.characteristics {
                        println!("  {:?}", characteristic);
                    }
                }
                if is_connected {
                    println!("Disconnecting from peripheral {:?}...", &local_name);
                    peripheral
                        .disconnect()
                        .await
                        .expect("Error disconnecting from BLE peripheral");
                }
            }
        }
    }
}