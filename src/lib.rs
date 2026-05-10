use std::error::Error;
use std::io::Read;

#[cfg(target_os = "linux")]
use std::time::Duration;
#[cfg(target_os = "linux")]
use bluetooth_serial_port::{scan_devices, BtAddr, BtProtocol, BtSocket};
#[cfg(target_os = "linux")]
use libc;

use chrono::{NaiveDateTime, NaiveTime};
use log::{debug, info};

pub mod divoom_file_format;
pub mod protocol;

use crate::protocol::alarm::Alarm;
use crate::protocol::animation::{Animation, ControlWord};
use crate::protocol::command::Command;
use crate::protocol::datetime::DateTime;
use crate::protocol::packet::Packet;

// ── macOS IOBluetooth FFI ────────────────────────────────────────────────────

#[cfg(target_os = "macos")]
#[no_mangle]
/// # Safety
/// `msg` must be a valid, non-null, null-terminated C string for the duration of this call.
pub unsafe extern "C" fn bt_log(level: u8, msg: *const std::os::raw::c_char) {
  let s = std::ffi::CStr::from_ptr(msg).to_string_lossy();
  match level {
    0 => log::debug!(target: "bt", "{}", s),
    1 => log::info!(target: "bt", "{}", s),
    _ => log::warn!(target: "bt", "{}", s),
  }
}

#[cfg(target_os = "macos")]
extern "C" {
  fn bt_rfcomm_send(
    addr: *const std::os::raw::c_char,
    packets: *const *const u8,
    sizes: *const u16,
    count: std::os::raw::c_int,
    delay_ms: std::os::raw::c_uint
  ) -> std::os::raw::c_int;

  fn bt_rfcomm_query(
    addr: *const std::os::raw::c_char,
    cmd: *const u8,
    cmd_len: u16,
    out: *mut u8,
    out_size: usize,
    out_len: *mut usize,
    timeout_ms: std::os::raw::c_uint
  ) -> std::os::raw::c_int;

  fn bt_list_devices();
}

// ── public types ─────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy)]
pub struct DeviceState {
  pub channel: u8,
  pub brightness: u8
}

// ── list_devices ─────────────────────────────────────────────────────────────

#[cfg(target_os = "linux")]
pub async fn list_devices() -> Result<(), Box<dyn Error>> {
  let duration = Duration::from_secs(20);
  info!("Scanning bluetooth devices for {:?}", duration);
  let devices = scan_devices(duration)?;
  info!("Found bluetooth devices {:?}", devices);
  Ok(())
}

#[cfg(target_os = "macos")]
pub async fn list_devices() -> Result<(), Box<dyn Error>> {
  info!("Listing paired Bluetooth devices");
  unsafe { bt_list_devices() };
  Ok(())
}

// ── send (private) ────────────────────────────────────────────────────────────

#[cfg(target_os = "linux")]
fn send(device: &str, packets: &[Packet]) -> Result<(), Box<dyn Error>> {
  use std::io::Write;
  use std::str::FromStr;
  let mac_address =
    BtAddr::from_str(device).map_err(|_| format!("Invalid MAC address: '{}'", device))?;
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

#[cfg(target_os = "macos")]
fn send(device: &str, packets: &[Packet]) -> Result<(), Box<dyn Error>> {
  use std::ffi::CString;
  use std::os::raw::{c_int, c_uint};

  info!("Connecting to {} via IOBluetooth RFCOMM", device);

  let addr = CString::new(device).map_err(|_| "Invalid device address string")?;

  let serialized: Vec<Vec<u8>> = packets
    .iter()
    .enumerate()
    .map(|(i, p)| {
      let bytes = p.serialize()?;
      debug!("  packet {}: {}", i + 1, hex::encode(&bytes));
      Ok(bytes)
    })
    .collect::<Result<_, Box<dyn Error>>>()?;

  let ptrs: Vec<*const u8> = serialized.iter().map(|b| b.as_ptr()).collect();
  let sizes: Vec<u16> = serialized.iter().map(|b| b.len() as u16).collect();

  let rc = unsafe {
    bt_rfcomm_send(
      addr.as_ptr(),
      ptrs.as_ptr(),
      sizes.as_ptr(),
      packets.len() as c_int,
      200 as c_uint
    )
  };

  if rc != 0 {
    return Err(format!("Bluetooth send failed (code {})", rc).into());
  }

  info!("All {} packet(s) sent", packets.len());
  Ok(())
}

// ── query (private, macOS only) ───────────────────────────────────────────────

#[cfg(target_os = "macos")]
fn query(device: &str, packet: &Packet, timeout_ms: u32) -> Result<Vec<u8>, Box<dyn Error>> {
  use std::ffi::CString;
  use std::os::raw::c_uint;

  let addr = CString::new(device).map_err(|_| "Invalid device address string")?;
  let serialized = packet.serialize()?;
  debug!("query packet: {}", hex::encode(&serialized));

  let mut out_buf = vec![0u8; 256];
  let mut out_len: usize = 0;

  let rc = unsafe {
    bt_rfcomm_query(
      addr.as_ptr(),
      serialized.as_ptr(),
      serialized.len() as u16,
      out_buf.as_mut_ptr(),
      out_buf.len(),
      &mut out_len,
      timeout_ms as c_uint
    )
  };

  if rc != 0 {
    return Err(format!("Bluetooth query failed (code {})", rc).into());
  }

  out_buf.truncate(out_len);
  debug!("query response: {}", hex::encode(&out_buf));
  Ok(out_buf)
}

// ── query (Linux) ─────────────────────────────────────────────────────────────

#[cfg(target_os = "linux")]
fn query(device: &str, packet: &Packet, timeout_ms: u32) -> Result<Vec<u8>, Box<dyn Error>> {
  use std::io::{Read, Write};
  use std::os::unix::io::AsRawFd;
  use std::str::FromStr;
  let mac = BtAddr::from_str(device).map_err(|_| format!("Invalid MAC address: '{}'", device))?;
  let mut socket = BtSocket::new(BtProtocol::RFCOMM)?;
  socket.connect(mac)?;
  info!("Connection successful, socket over RFCOMM/SPP acquired");
  let secs = (timeout_ms / 1000) as libc::time_t;
  let usecs = ((timeout_ms % 1000) * 1000) as libc::suseconds_t;
  let tv = libc::timeval { tv_sec: secs, tv_usec: usecs };
  unsafe {
    libc::setsockopt(
      socket.as_raw_fd(),
      libc::SOL_SOCKET,
      libc::SO_RCVTIMEO,
      &tv as *const _ as *const libc::c_void,
      std::mem::size_of::<libc::timeval>() as libc::socklen_t,
    );
  }
  let serialized = packet.serialize()?;
  debug!("query packet: {}", hex::encode(&serialized));
  socket.write_all(&serialized)?;
  let mut buf = Vec::new();
  let mut byte = [0u8; 1];
  loop {
    match socket.read(&mut byte) {
      Ok(0) => break,
      Ok(_) => {
        buf.push(byte[0]);
        if byte[0] == 0x02 {
          break;
        }
      }
      Err(e)
        if e.kind() == std::io::ErrorKind::WouldBlock
          || e.kind() == std::io::ErrorKind::TimedOut =>
      {
        break
      }
      Err(e) => return Err(e.into()),
    }
  }
  debug!("query response: {}", hex::encode(&buf));
  Ok(buf)
}

// ── response parsing ──────────────────────────────────────────────────────────

fn parse_settings_response(bytes: &[u8]) -> Result<DeviceState, Box<dyn Error>> {
  // Response layout (confirmed from Ditoo Pro packet capture):
  //   01 LL LL 04 46 55 [channel] [13 unknown bytes] [brightness] ... CRC CRC 02
  //   idx:              6         7-19                19
  if bytes.len() < 20 {
    return Err(format!("Response too short ({} bytes): {}", bytes.len(), hex::encode(bytes)).into());
  }
  if bytes[0] != 0x01 {
    return Err(format!("Bad start byte: {:02X}", bytes[0]).into());
  }
  if bytes[3] != 0x04 {
    return Err(format!("Expected response marker 0x04, got {:02X}", bytes[3]).into());
  }
  if bytes[4] != 0x46 {
    return Err(format!("Expected command echo 0x46, got {:02X}", bytes[4]).into());
  }
  if bytes[5] != 0x55 {
    return Err(format!("Expected fixed byte 0x55, got {:02X}", bytes[5]).into());
  }
  Ok(DeviceState {
    channel: bytes[6],
    brightness: bytes[19]
  })
}

// ── helpers ───────────────────────────────────────────────────────────────────

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

// ── public API ────────────────────────────────────────────────────────────────

pub async fn get_state(device: &str) -> Result<DeviceState, Box<dyn Error>> {
  let packet = Packet {
    command: Command::GetSettings,
    payload: vec![]
  };
  let response = query(device, &packet, 3000)?;
  parse_settings_response(&response)
}

pub async fn send_set_channel(device: &str, channel: u8) -> Result<(), Box<dyn Error>> {
  let packet = Packet {
    command: Command::SetChannel,
    payload: vec![channel]
  };
  send(device, &[packet])
}

pub async fn send_alarm(device: &str) -> Result<(), Box<dyn Error>> {
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
  send(device, &[packet])
}

pub async fn send_divoom_animation<R: Read>(
  device: &str,
  reader: &mut R
) -> Result<(), Box<dyn Error>> {
  let mut animation = Vec::new();
  reader.read_to_end(&mut animation)?;
  let packets = create_network_packets_from(&animation)?;
  send(device, &packets)?;
  Ok(())
}

pub async fn send_set_datetime(
  device: &str,
  datetime: NaiveDateTime
) -> Result<(), Box<dyn Error>> {
  let payload = DateTime { datetime };
  let packet = Packet {
    command: Command::SetDateTime,
    payload: payload.serialize()?
  };
  send(device, &[packet])
}
