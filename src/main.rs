use std::error::Error;
use std::fs::File;
use std::io::BufReader;
use std::io::Read;
use std::io::Write;
use std::str::FromStr;
use std::time::Duration;

use bluetooth_serial_port::{scan_devices, BtAddr, BtProtocol, BtSocket};
use byteorder::LittleEndian;
use byteorder::ReadBytesExt;
use clap::{Parser, Subcommand};
use env_logger::{Builder, Env};
use image::{DynamicImage, RgbImage};
use log::info;

use Command::{ListDevices, Send, DebugImage};
use SendCommand::Alert;

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
    // BtAddr uses FromStr -> Err<()>, which doesn't work with clap:
    // https://github.com/clap-rs/clap/issues/5360
    Send {
        mac_address: String,
        #[command(subcommand)]
        send: SendCommand,
    },

    /// Show detailed information about an image in Divoom file format
    DebugImage {
        filename: String
    }
}

#[derive(Subcommand, Debug)]
enum SendCommand {
    Alert {
        #[arg(required = true, number_of_values = 1, value_parser = clap::builder::BoolishValueParser::new())]
        enable: bool,
    },
}

async fn list_devices() -> Result<(), Box<dyn Error>> {
    let duration = Duration::from_secs(20);
    info!("Scanning bluetooth devices for {:?}", duration);
    let devices = scan_devices(duration)?;
    info!("Found bluetooth devices {:?}", devices);

    Ok(())
}

async fn send_command(mac_address: BtAddr) -> Result<(), Box<dyn Error>> {
    info!("Connecting to device with MAC address {:?}", mac_address);

    let mut socket = BtSocket::new(BtProtocol::RFCOMM)?;
    socket.connect(mac_address)?;
    info!("Connection successful, socket over RFCOMM/SPP acquired");

    info!("Sending message..");
    let message = hex::decode("010d00430000142e000200000028bc0002")?;
    let num_bytes_written = socket.write(&message)?;
    info!(
        "Wrote {}/{} bytes ({}%)",
        num_bytes_written,
        message.len(),
        num_bytes_written * 100 / message.len()
    );

    Ok(())
}

#[derive(Debug)]
#[allow(dead_code)]
struct FrameHeader {
    magic_number: u8,
    length: u16,
    time_in_milliseconds: u16,
    reuse_palette: bool,
    color_count: u8
}

impl FrameHeader {
    fn from_reader<R: Read>(reader: &mut R) -> std::io::Result<Self> {
        let magic_number = reader.read_u8()?;
        let length = reader.read_u16::<LittleEndian>()?;
        let time_in_milliseconds = reader.read_u16::<LittleEndian>()?;
        let reuse_palette = reader.read_u8()? != 0;
        let color_count = reader.read_u8()?;

        Ok(FrameHeader {
            magic_number,
            length,
            time_in_milliseconds,
            reuse_palette,
            color_count,
        })
    }
}

fn read_divoom_16x16_image<R: Read>(reader: &mut R) -> Result<DynamicImage, Box<dyn Error>> {
    let image = RgbImage::new(16, 16);

    let frame_header = FrameHeader::from_reader(reader)?;
    info!("Image frame header: {:?}", frame_header);

    if frame_header.magic_number != 0xAA {
        return Err(format!("Magic number does not match {:#04X}: {:#04X}", 0xAA, frame_header.magic_number).into())
    }

    Ok(DynamicImage::ImageRgb8(image))
}

fn read_divoom_16x16_image_from_file(filename: String) -> Result<DynamicImage, Box<dyn Error>> {
    let file = File::open(filename)?;
    let mut reader = BufReader::new(file);
    read_divoom_16x16_image(&mut reader)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    Builder::from_env(Env::default().default_filter_or("debug")).init();

    let args = Args::parse();

    match args.command {
        ListDevices => list_devices().await?,
        Send { mac_address, send } => match send {
            Alert { enable } => {
                match enable {
                    true => info!("Enabling alert.."),
                    false => info!("Disabling alert.."),
                }
                send_command(
                    BtAddr::from_str(&mac_address)
                        .map_err(|_| format!("Invalid MAC address: '{}'", mac_address))?,
                )
                .await?
            }
        },
        DebugImage { filename } => {
            read_divoom_16x16_image_from_file(filename)?;
        }
    }

    Ok(())
}
