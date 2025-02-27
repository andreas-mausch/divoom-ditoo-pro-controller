use std::error::Error;
use std::io::{BufWriter, Read, Write};
use std::time::Duration;

use bluetooth_serial_port::{scan_devices, BtAddr, BtProtocol, BtSocket};
use byteorder::{LittleEndian, WriteBytesExt};
use log::{debug, info};

pub mod protocol;

use protocol::Packet;

pub async fn list_devices() -> Result<(), Box<dyn Error>> {
    let duration = Duration::from_secs(20);
    info!("Scanning bluetooth devices for {:?}", duration);
    let devices = scan_devices(duration)?;
    info!("Found bluetooth devices {:?}", devices);

    Ok(())
}

pub async fn send_command(mac_address: BtAddr) -> Result<(), Box<dyn Error>> {
    let packet = Packet::from(0x43, &hex::decode("0000142e000200000028")?)?;
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
    packets.push(Packet::from(139, &hex::decode("00b4010000")?)?);

    let mut animation_packets = animation
        .chunks(256)
        .enumerate()
        .map(|(index, chunk)| {
            let payload_size = chunk.len() + 7;
            let mut payload = Vec::<u8>::with_capacity(payload_size);
            {
                let mut writer = BufWriter::new(&mut payload);
                writer.write_u8(1)?;
                writer.write_u32::<LittleEndian>(animation.len() as u32)?;
                writer.write_u16::<LittleEndian>(index as u16)?;
                writer.write_all(chunk)?;
            }
            Packet::from(139, &payload)
        })
        .collect::<Result<Vec<_>, Box<dyn Error>>>()?;
    packets.append(&mut animation_packets);

    Ok(packets)
}

pub fn send_divoom_animation<R: Read>(
    mac_address: BtAddr,
    reader: &mut R,
) -> Result<(), Box<dyn Error>> {
    let mut animation = Vec::new();
    reader.read_to_end(&mut animation)?;

    let packets = create_network_packets_from(&animation)?;
    send(
        mac_address,
        &packets,
    )?;
    Ok(())
}
