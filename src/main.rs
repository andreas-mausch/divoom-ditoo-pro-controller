use std::error::Error;
use std::time::Duration;

use btleplug::api::{BDAddr, Central, CentralEvent, Manager as _, Peripheral, ScanFilter};
use btleplug::platform::Manager;
use CentralEvent::DeviceDiscovered;
use clap::{Parser, Subcommand};
use env_logger::{Builder, Env};
use futures::stream::StreamExt;
use log::info;
use tokio::time;

use Command::{ListDevices, SendCommand};

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
    ListDevices,

    /// Connects to a Divoom via it's MAC address and sends a command
    SendCommand {
        mac_address: BDAddr
    },
}

async fn list_devices() -> Result<(), Box<dyn Error>> {
    let manager = Manager::new().await?;

    // get the first bluetooth adapter
    let adapters = manager.adapters().await?;
    let adapter = adapters.first().ok_or("No bluetooth adapter found")?;

    // start scanning for devices
    adapter.start_scan(ScanFilter::default()).await?;
    // instead of waiting, you can use central.events() to get a stream which will
    // notify you of new devices, for an example of that see examples/event_driven_discovery.rs
    time::sleep(Duration::from_secs(2)).await;

    for peripheral in adapter.peripherals().await? {
        let properties = peripheral.properties().await?.ok_or("Could not get properties for bluetooth device")?;
        info!("Found bluetooth device: {:?} {:?}", peripheral.address(), properties.local_name);
    }

    Ok(())
}

async fn send_command(mac_address: BDAddr) -> Result<(), Box<dyn Error>> {
    info!("Connecting to device {:?}", mac_address);

    let manager = Manager::new().await?;
    let adapters = manager.adapters().await?;
    let adapter = adapters.first().ok_or("No bluetooth adapter found")?;

    let mut events = adapter.events().await?;
    adapter.start_scan(ScanFilter::default()).await?;

    let peripheral = loop {
        match events.next().await {
            Some(DeviceDiscovered(id)) => {
                let discovered_mac_address = adapter.peripheral(&id).await?.address();
                info!("DeviceDiscovered: {}", discovered_mac_address);

                if discovered_mac_address == mac_address {
                    break Some(adapter.peripheral(&id).await?);
                }
            }
            Some(_) => {}
            None => break None
        }
    }.ok_or("Device not found")?;

    peripheral.connect().await?;
    peripheral.discover_services().await?;

    info!("Connected to device with MAC address {}", mac_address);

    peripheral.disconnect().await?;

    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    Builder::from_env(Env::default().default_filter_or("debug")).init();

    let args = Args::parse();

    match args.command {
        ListDevices => list_devices().await?,
        SendCommand { mac_address } => send_command(mac_address).await?
    }

    Ok(())
}
