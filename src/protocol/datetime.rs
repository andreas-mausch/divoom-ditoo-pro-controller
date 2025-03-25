use std::error::Error;
use std::io::BufWriter;

use byteorder::WriteBytesExt;
use chrono::{Datelike, NaiveDateTime, Timelike};

#[derive(Debug)]
pub struct DateTime {
  pub datetime: NaiveDateTime
}

impl DateTime {
  pub fn serialize(&self) -> Result<Vec<u8>, Box<dyn Error>> {
    let mut buffer = Vec::<u8>::new();

    // This command is not documented, but I found it here:
    // https://github.com/d03n3rfr1tz3/esp32-divoom/blob/5e42ee3ca37d767dfcbb47fba66d3bd3f20bae33/src/divoom/divoom.cpp#L579
    let mut writer = BufWriter::new(&mut buffer);
    // Indicates which alarm to set, starting from 0.
    writer.write_u8((self.datetime.year() % 100) as u8)?;
    writer.write_u8((self.datetime.year() / 100) as u8)?;
    writer.write_u8(self.datetime.month() as u8)?;
    writer.write_u8(self.datetime.day() as u8)?;
    writer.write_u8(self.datetime.hour() as u8)?;
    writer.write_u8(self.datetime.minute() as u8)?;
    writer.write_u8(self.datetime.second() as u8)?;
    drop(writer);

    Ok(buffer)
  }
}
