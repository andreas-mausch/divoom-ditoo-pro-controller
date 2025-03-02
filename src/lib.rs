use std::error::Error;
use std::io::{Read, Write};
use std::time::Duration;

use bluetooth_serial_port::{scan_devices, BtAddr, BtProtocol, BtSocket};
use chrono::NaiveTime;
use log::{debug, info};

pub mod divoom_file_format;
pub mod protocol;

use crate::protocol::alarm::Alarm;
use crate::protocol::animation::{Animation, ControlWord};
use crate::protocol::command::Command;
use crate::protocol::packet::Packet;

pub async fn list_devices() -> Result<(), Box<dyn Error>> {
  let duration = Duration::from_secs(20);
  info!("Scanning bluetooth devices for {:?}", duration);
  let devices = scan_devices(duration)?;
  info!("Found bluetooth devices {:?}", devices);

  Ok(())
}

pub async fn send_command(mac_address: BtAddr) -> Result<(), Box<dyn Error>> {
  let alarm = Alarm {
    index: 0,
    enable: false,
    time: NaiveTime::from_hms_opt(13, 37, 0).ok_or("Invalid time")?,
    repeat: 0,
    mode: 0,
    trigger_mode: 0,
    fm: [0, 0],
    volume: 100
  };
  let packet = Packet {
    command: Command::Alarm,
    payload: alarm.serialize()?
  };
  send(mac_address, &[packet])
}

fn send(mac_address: BtAddr, packets: &[Packet]) -> Result<(), Box<dyn Error>> {
  info!("Connecting to device with MAC address {:?}", mac_address);

  let mut socket = BtSocket::new(BtProtocol::RFCOMM)?;
  socket.connect(mac_address)?;
  info!("Connection successful, socket over RFCOMM/SPP acquired");

  packets
    .iter()
    .enumerate()
    .try_for_each(|(index, packet)| -> Result<(), Box<dyn Error>> {
      info!("Sending packet {}/{}..", index + 1, packets.len());
      let serialized = packet.serialize()?;
      debug!("  {}", hex::encode(&serialized));

      let num_bytes_written = socket.write(&serialized)?;
      info!(
        "  Wrote {}/{} bytes ({}%)",
        num_bytes_written,
        serialized.len(),
        num_bytes_written * 100 / serialized.len()
      );
      std::thread::sleep(std::time::Duration::from_millis(200));

      Ok(())
    })?;

  Ok(())
}

fn create_network_packets_from(animation: &[u8]) -> Result<Vec<Packet>, Box<dyn Error>> {
  let mut packets = Vec::<Packet>::new();
  packets.push(Packet {
    command: Command::Animation,
    payload: Animation {
      control_word: ControlWord::StartSeeding,
      file_size: animation.len() as u32,
      offset_id: 0,
      image_part: Vec::new()
    }
    .serialize()?
  });

  let mut animation_packets = animation
    .chunks(256)
    .enumerate()
    .map(|(index, chunk)| {
      Ok(Packet {
        command: Command::Animation,
        payload: Animation {
          control_word: ControlWord::SendingData,
          file_size: animation.len() as u32,
          offset_id: index as u16,
          image_part: chunk.to_vec()
        }
        .serialize()?
      })
    })
    .collect::<Result<Vec<_>, Box<dyn Error>>>()?;
  packets.append(&mut animation_packets);

  Ok(packets)
}

pub fn send_divoom_animation<R: Read>(
  mac_address: BtAddr,
  reader: &mut R
) -> Result<(), Box<dyn Error>> {
  let mut animation = Vec::new();
  reader.read_to_end(&mut animation)?;

  let packets = create_network_packets_from(&animation)?;
  send(mac_address, &packets)?;
  Ok(())
}
