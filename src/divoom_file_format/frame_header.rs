use std::error::Error;
use std::io::Read;

use byteorder::{LittleEndian, ReadBytesExt};

#[derive(Debug)]
pub struct FrameHeader {
  pub time_in_milliseconds: u16,
  pub reuse_palette: bool,
  pub color_count: u8
}

pub const FRAME_HEADER_MAGIC_NUMBER: u8 = 0xAA;

impl FrameHeader {
  pub fn from_reader<R: Read>(reader: &mut R) -> Result<Self, Box<dyn Error>> {
    let magic_number = reader.read_u8()?;
    reader.read_u16::<LittleEndian>()?; // length
    let time_in_milliseconds = reader.read_u16::<LittleEndian>()?;
    let reuse_palette = reader.read_u8()? != 0;
    let color_count = reader.read_u8()?;

    if magic_number != FRAME_HEADER_MAGIC_NUMBER {
      return Err(
        format!(
          "Magic number does not match {:#04X}: {:#04X}",
          FRAME_HEADER_MAGIC_NUMBER, magic_number
        )
        .into()
      );
    }

    Ok(FrameHeader {
      time_in_milliseconds,
      reuse_palette,
      color_count
    })
  }
}
