use std::error::Error;
use std::io::{BufWriter, Write};

use byteorder::{LittleEndian, WriteBytesExt};

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ControlWord {
  StartSeeding = 0,
  SendingData = 1,
  TerminateSending = 2
}

#[derive(Debug)]
pub struct Animation {
  pub control_word: ControlWord,
  pub file_size: u32,
  pub offset_id: u16,
  pub image_part: Vec<u8>
}

impl Animation {
  pub fn serialize(&self) -> Result<Vec<u8>, Box<dyn Error>> {
    let mut buffer = Vec::<u8>::new();

    // https://docin.divoom-gz.com/web/#/5/293
    let mut writer = BufWriter::new(&mut buffer);
    writer.write_u8(self.control_word as u8)?;

    if self.control_word != ControlWord::TerminateSending {
      // Total size of the file in little-endian format
      writer.write_u32::<LittleEndian>(self.file_size)?;
    }

    if self.control_word == ControlWord::SendingData {
      // Little-endian value starting from 0
      writer.write_u16::<LittleEndian>(self.offset_id)?;
      // Actual data to be sent (up to 256 bytes)
      writer.write_all(&self.image_part)?;
    }
    drop(writer);

    Ok(buffer)
  }
}
