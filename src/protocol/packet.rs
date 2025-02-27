use std::error::Error;
use std::io::{BufWriter, Write};

use byteorder::{LittleEndian, WriteBytesExt};

use super::command::Command;

#[derive(Debug)]
pub struct Packet {
    pub command: Command,
    pub payload: Vec<u8>
}

impl Packet {
    pub fn serialize(&self) -> Result<Vec<u8>, Box<dyn Error>> {
        let mut buffer = Vec::<u8>::new();
        let mut writer = BufWriter::new(&mut buffer);
        // start, always 1
        writer.write_u8(1)?;
        // length
        writer.write_u16::<LittleEndian>(self.payload.len() as u16 + 3)?;
        writer.write_u8(self.command.value())?;
        writer.write_all(&self.payload)?;
        writer.write_u16::<LittleEndian>(self.checksum()?)?;
        // end, always 2
        writer.write_u8(2)?;
        drop(writer);

        Ok(buffer)
    }

    fn checksum(&self) -> Result<u16, Box<dyn Error>> {
        let mut buffer = Vec::<u8>::new();
        let mut writer = BufWriter::new(&mut buffer);
        writer.write_u16::<LittleEndian>(self.payload.len() as u16 + 3)?;
        writer.write_u8(self.command.value())?;
        writer.write_all(&self.payload)?;
        drop(writer);

        Ok(Self::checksum_from_buffer(&buffer))
    }

    fn checksum_from_buffer(buffer: &[u8]) -> u16 {
        buffer.iter().fold(0u16, |acc, x| acc + *x as u16)
    }
}
