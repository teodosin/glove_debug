use bevy::prelude::*;

use dbus::blocking::Connection;
use std::time::Duration;

extern crate hidapi;

use hidapi::HidApi;

mod ble;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, connect)
        .add_systems(Update, listen)
    .run();
}

fn connect() {
    // do the bluetooth connection thingy

    match HidApi::new() {
        Ok(api) => {
            for device in api.device_list() {
                println!("-------------------------------");
                println!("{:04x}:{:04x}", device.vendor_id(), device.product_id());
                println!("{:?}", device.path());
                println!("{:?}", device.serial_number());
                println!("{:?}", device.product_string().unwrap());
            }
        },
        Err(e) => {
            eprintln!("Error: {}", e);
        },
    }

    println!("-------------------- Hidapi before, dbus after ------------------");

    let conn = Connection::new_session().unwrap();

    // Second, create a wrapper struct around the connection that makes it easy
    // to send method calls to a specific destination and path.
    let proxy = conn.with_proxy("org.freedesktop.DBus", "/", Duration::from_millis(5000));

    // Now make the method call. The ListNames method call takes zero input parameters and
    // one output parameter which is an array of strings.
    // Therefore the input is a zero tuple "()", and the output is a single tuple "(names,)".
    let (names,): (Vec<String>,) = proxy.method_call("org.freedesktop.DBus", "ListNames", ()).unwrap();

    // Let's print all the names to stdout.
    for name in names { println!("{}", name); }
}

fn listen() {
    // nothing right now
}