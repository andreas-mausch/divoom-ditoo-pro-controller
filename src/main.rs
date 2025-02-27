use std::error::Error;
use std::fs::File;
use std::str::FromStr;

use bluetooth_serial_port::BtAddr;
use clap::{Parser, Subcommand};
use env_logger::{Builder, Env};
use log::info;

use Command::{DebugImage, ListDevices, Send};
use SendCommand::{Alarm, Animation};

use divoom_ditoo_pro_controller::{list_devices, send_command, send_divoom_animation};

pub mod divoom_file_format;

use crate::divoom_file_format::read_divoom_16x16_image_from_file;

/// CLI tool to send bluetooth commands to a Divoom Ditoo Pro
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Args {
  #[command(subcommand)]
  command: Command
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
    send: SendCommand
  },

  /// Show detailed information about an image in Divoom file format
  DebugImage { filename: String }
}

#[derive(Subcommand, Debug)]
enum SendCommand {
  Alarm {
    #[arg(required = true, number_of_values = 1, value_parser = clap::builder::BoolishValueParser::new())]
    enable: bool
  },
  Animation {
    filename: String
  }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
  Builder::from_env(Env::default().default_filter_or("debug")).init();

  let args = Args::parse();

  match args.command {
    ListDevices => list_devices().await?,
    Send { mac_address, send } => match send {
      Alarm { enable } => {
        match enable {
          true => info!("Enabling alarm.."),
          false => info!("Disabling alarm..")
        }
        send_command(
          BtAddr::from_str(&mac_address)
            .map_err(|_| format!("Invalid MAC address: '{}'", mac_address))?
        )
        .await?
      }
      Animation { filename } => {
        let mut file = File::open(filename)?;
        send_divoom_animation(
          BtAddr::from_str(&mac_address)
            .map_err(|_| format!("Invalid MAC address: '{}'", mac_address))?,
          &mut file
        )?;
      }
    },
    DebugImage { filename } => {
      read_divoom_16x16_image_from_file(filename)?;
    }
  }

  Ok(())
}
