use std::error::Error;
use std::io::Write;
use std::time::Duration;

use bluetooth_serial_port::{scan_devices, BtAddr, BtProtocol, BtSocket};
use log::info;

pub async fn list_devices() -> Result<(), Box<dyn Error>> {
  let duration = Duration::from_secs(20);
  info!("Scanning bluetooth devices for {:?}", duration);
  let devices = scan_devices(duration)?;
  info!("Found bluetooth devices {:?}", devices);

  Ok(())
}

pub async fn send_command(mac_address: BtAddr) -> Result<(), Box<dyn Error>> {
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
