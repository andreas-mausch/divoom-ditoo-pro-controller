use std::io::Read;

use byteorder::{LittleEndian, ReadBytesExt};

#[derive(Debug)]
#[allow(dead_code)]
pub struct FrameHeader {
    pub magic_number: u8,
    length: u16,
    time_in_milliseconds: u16,
    reuse_palette: bool,
    pub color_count: u8,
}

impl FrameHeader {
    pub fn from_reader<R: Read>(reader: &mut R) -> std::io::Result<Self> {
        let magic_number = reader.read_u8()?;
        let length = reader.read_u16::<LittleEndian>()?;
        let time_in_milliseconds = reader.read_u16::<LittleEndian>()?;
        let reuse_palette = reader.read_u8()? != 0;
        let color_count = reader.read_u8()?;

        Ok(FrameHeader {
            magic_number,
            length,
            time_in_milliseconds,
            reuse_palette,
            color_count,
        })
    }
}
