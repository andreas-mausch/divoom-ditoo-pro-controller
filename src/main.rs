use std::error::Error;
use std::time::Duration;

use btleplug::api::{Central, Manager as _, Peripheral, ScanFilter};
use btleplug::platform::Manager;
use clap::{Parser, Subcommand};
use env_logger::{Builder, Env};
use log::info;
use tokio::time;

/// CLI tool to send bluetooth commands to a Divoom Ditoo Pro
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand, Debug)]
enum Command {
    /// Lists all available bluetooth devices and tries to find a Divoom
    ListDevices
}

async fn list_devices() -> Result<(), Box<dyn Error>> {
    let manager = Manager::new().await.unwrap();

    // get the first bluetooth adapter
    let adapters = manager.adapters().await?;
    let central = adapters.into_iter().nth(0).unwrap();

    // start scanning for devices
    central.start_scan(ScanFilter::default()).await?;
    // instead of waiting, you can use central.events() to get a stream which will
    // notify you of new devices, for an example of that see examples/event_driven_discovery.rs
    time::sleep(Duration::from_secs(2)).await;

    for peripheral in central.peripherals().await.unwrap() {
        let properties = peripheral.properties().await.unwrap().unwrap();
        info!("Found bluetooth device: {:?} {:?}", peripheral.address(), properties.local_name);
    }

    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    Builder::from_env(Env::default().default_filter_or("debug")).init();

    let args = Args::parse();

    match args.command {
        Command::ListDevices => list_devices().await?
    }

    Ok(())
}
