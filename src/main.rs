use std::error::Error;
use std::fs::File;
use std::io::{BufReader, BufWriter};

use chrono::NaiveDateTime;
use clap::{Parser, Subcommand};
use env_logger::{Builder, Env};
use log::info;

use Command::{Convert, DebugImage, ListDevices, Send};

use divoom_ditoo_pro_controller::divoom_file_format::animation::Animation;
use divoom_ditoo_pro_controller::divoom_file_format::frame::bits_per_pixel;
use divoom_ditoo_pro_controller::{
  get_state, list_devices, send_alarm, send_divoom_animation, send_set_channel, send_set_datetime
};

/// CLI tool to send bluetooth commands to a Divoom Ditoo Pro
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Args {
  #[command(subcommand)]
  command: Command
}

#[derive(Subcommand, Debug)]
enum Command {
  /// Lists available Bluetooth devices (Linux: scans 20s; macOS: lists paired devices)
  ListDevices,

  /// Connects to a Divoom and sends a command.
  /// On Linux: pass the MAC address (AA:BB:CC:DD:EE:FF).
  /// On macOS: pass the address from list-devices (e.g. aa-bb-cc-dd-ee-ff).
  Send {
    device: String,
    #[command(subcommand)]
    send: SendCommand
  },

  /// Converts a Divoom animation to GIF and vice versa
  Convert {
    #[command(subcommand)]
    convert: ConvertCommand
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
  },
  /// Query current display state (channel and brightness)
  GetSettings,
  /// Switch display channel: 0=Clock, 1=Cloud, 2=Equalizer, 3=Custom, 4=Scoreboard, 5=Stopwatch
  SetChannel {
    channel: u8
  },
  SetDateTime {
    datetime: NaiveDateTime
  }
}

#[derive(Subcommand, Debug)]
enum ConvertCommand {
  ToGif {
    input_filename: String,
    output_filename: String
  },
  ToDivoom16 {
    input_filename: String,
    output_filename: String
  }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
  Builder::from_env(Env::default().default_filter_or("debug")).init();

  let args = Args::parse();

  match args.command {
    ListDevices => list_devices().await?,
    Send { device, send } => match send {
      SendCommand::Alarm { enable } => {
        match enable {
          true => info!("Enabling alarm.."),
          false => info!("Disabling alarm..")
        }
        send_alarm(&device).await?
      }
      SendCommand::Animation { filename } => {
        let mut file = File::open(filename)?;
        send_divoom_animation(&device, &mut file).await?;
      }
      SendCommand::GetSettings => {
        let state = get_state(&device).await?;
        println!("channel={} brightness={}", state.channel, state.brightness);
      }
      SendCommand::SetChannel { channel } => {
        send_set_channel(&device, channel).await?
      }
      SendCommand::SetDateTime { datetime } => {
        send_set_datetime(&device, datetime).await?
      }
    },
    Convert { convert } => match convert {
      ConvertCommand::ToGif {
        input_filename,
        output_filename
      } => {
        let animation = Animation::from_16x16(&mut BufReader::new(File::open(input_filename)?))?;
        animation.save_to_gif(&mut BufWriter::new(File::create(output_filename)?))?;
      }
      ConvertCommand::ToDivoom16 {
        input_filename,
        output_filename
      } => {
        let animation = Animation::from_gif(&mut BufReader::new(File::open(input_filename)?))?;
        animation.save_to_divoom_format(&mut BufWriter::new(File::create(output_filename)?))?;
      }
    },
    DebugImage { filename } => {
      use log::debug;
      let animation = Animation::from_16x16(&mut BufReader::new(File::open(filename)?))?;
      animation
        .frames
        .iter()
        .enumerate()
        .for_each(|(index, frame)| {
          let bpp = bits_per_pixel(frame.palette.len() as u32);
          let pixel_data_in_bits = 16 * 16 * bpp as u32;
          let pixel_data_in_bytes = pixel_data_in_bits.div_ceil(8);
          debug!("Frame #{}", index);
          debug!(
            "  Pixel data size: {} bits = {} bytes",
            pixel_data_in_bits, pixel_data_in_bytes
          );
          debug!("  {:?}", frame.header);
          debug!(
            "  Color count: {} / Bits per pixel: {}",
            frame.palette.len(),
            bpp
          );
          debug!(
            "  Local palette: {:?}",
            frame
              .local_palette
              .iter()
              .map(|color| format!("#{:02X}{:02X}{:02X}", color[0], color[1], color[2]))
              .collect::<Vec<_>>()
          );
        })
    }
  }

  Ok(())
}
