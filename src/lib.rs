use std::error::Error;
use std::io::{BufWriter, Read, Write};
use std::time::Duration;

use bluetooth_serial_port::{scan_devices, BtAddr, BtProtocol, BtSocket};
use byteorder::{LittleEndian, WriteBytesExt};
use log::info;

pub async fn list_devices() -> Result<(), Box<dyn Error>> {
    let duration = Duration::from_secs(20);
    info!("Scanning bluetooth devices for {:?}", duration);
    let devices = scan_devices(duration)?;
    info!("Found bluetooth devices {:?}", devices);

    Ok(())
}

pub async fn send_command(mac_address: BtAddr) -> Result<(), Box<dyn Error>> {
    let packet = hex::decode("010d00430000142e000200000028bc0002")?;
    send(mac_address, &[packet.as_slice()])
}

fn send(mac_address: BtAddr, packets: &[&[u8]]) -> Result<(), Box<dyn Error>> {
    info!("Connecting to device with MAC address {:?}", mac_address);

    let mut socket = BtSocket::new(BtProtocol::RFCOMM)?;
    socket.connect(mac_address)?;
    info!("Connection successful, socket over RFCOMM/SPP acquired");

    packets
        .iter()
        .enumerate()
        .try_for_each(|(index, packet)| -> Result<(), Box<dyn Error>> {
            info!("Sending packet {}/{}..", index + 1, packets.len());

            let num_bytes_written = socket.write(packet)?;
            info!(
                "  Wrote {}/{} bytes ({}%)",
                num_bytes_written,
                packet.len(),
                num_bytes_written * 100 / packet.len()
            );

            Ok(())
        })?;

    Ok(())
}

fn checksum(buffer: &[u8]) -> u16 {
    buffer.iter().fold(0u16, |acc, x| acc + *x as u16)
}

fn create_network_packet(command: u8, payload: &[u8]) -> Result<Vec<u8>, Box<dyn Error>> {
    let packet_size = (payload.len() + 7) as u16;
    let mut packet = Vec::<u8>::with_capacity(packet_size as usize);

    {
        let mut writer = BufWriter::new(&mut packet);
        writer.write_u8(1)?;
        writer.write_u16::<LittleEndian>(packet_size - 4)?;
        writer.write_u8(command)?;
        writer.write_all(payload)?;
        writer.flush()?;
    }

    let checksum = checksum(&packet[..packet_size as usize - 3]);

    {
        let mut writer = BufWriter::new(&mut packet);
        writer.write_u16::<LittleEndian>(checksum)?;
        writer.write_u8(2)?;
        writer.flush()?;
    }

    Ok(packet)
}

fn create_network_packets_from(animation: &[u8]) -> Result<Vec<Vec<u8>>, Box<dyn Error>> {
    let mut packets = Vec::<Vec<u8>>::new();
    packets.push(create_network_packet(139, &hex::decode("00b4010000")?)?);

    let mut xxx = animation
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
            create_network_packet(139, &payload)
        })
        .collect::<Result<Vec<_>, Box<dyn Error>>>()?;
    packets.append(&mut xxx);

    info!("Packets; {:?}", packets);
    Ok(packets)
}

pub fn send_divoom_animation<R: Read>(
    mac_address: BtAddr,
    reader: &mut R,
) -> Result<(), Box<dyn Error>> {
    let mut animation = Vec::new();
    reader.read_to_end(&mut animation)?;
    info!("{:?}", animation);
    let packets = create_network_packets_from(&animation)?;
    send(
        mac_address,
        &packets
            .iter()
            .map(|packet| packet.as_slice())
            .collect::<Vec<&[u8]>>(),
    )?;
    Ok(())
}
