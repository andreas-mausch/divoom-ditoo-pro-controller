use std::error::Error;
use std::io::{BufWriter, Write};

use byteorder::{LittleEndian, WriteBytesExt};

use super::command::Command;

#[derive(Debug)]
pub struct Packet {
    pub start: u8,
    pub length: u16,
    pub command: Command,
    pub payload: Vec<u8>,
    pub checksum: u16,
    pub end: u8,
}

impl Packet {
    pub fn from(command: Command, payload: &[u8]) -> Result<Self, Box<dyn Error>> {
        Ok(Packet {
            start: 1,
            length: payload.len() as u16 + 3,
            command,
            payload: payload.to_vec(),
            checksum: Self::checksum(command, payload)?,
            end: 2,
        })
    }

    pub fn serialize(&self) -> Result<Vec<u8>, Box<dyn Error>> {
        let mut buffer = Vec::<u8>::new();

        {
            let mut writer = BufWriter::new(&mut buffer);
            writer.write_u8(self.start)?;
            writer.write_u16::<LittleEndian>(self.length)?;
            writer.write_u8(self.command.value())?;
            writer.write_all(&self.payload)?;
            writer.write_u16::<LittleEndian>(self.checksum)?;
            writer.write_u8(self.end)?;
            writer.flush()?;
        }

        Ok(buffer)
    }

    fn checksum(command: Command, payload: &[u8]) -> Result<u16, Box<dyn Error>> {
        let mut buffer = Vec::<u8>::new();

        {
            let mut writer = BufWriter::new(&mut buffer);
            writer.write_u16::<LittleEndian>(payload.len() as u16 + 3)?;
            writer.write_u8(command.value())?;
            writer.write_all(payload)?;
            writer.flush()?;
        }

        Ok(Self::checksum_from_buffer(&buffer))
    }

    fn checksum_from_buffer(buffer: &[u8]) -> u16 {
        buffer.iter().fold(0u16, |acc, x| acc + *x as u16)
    }
}
