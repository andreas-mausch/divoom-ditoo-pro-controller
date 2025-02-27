use std::error::Error;
use std::io::{BufWriter, Write};

use byteorder::WriteBytesExt;
use chrono::{NaiveTime, Timelike};

#[derive(Debug)]
pub struct Alarm {
  pub index: u8,
  pub enable: bool,
  pub time: NaiveTime,
  pub repeat: u8,
  pub mode: u8,
  pub trigger_mode: u8,
  pub fm: [u8; 2],
  pub volume: u8
}

impl Alarm {
  pub fn serialize(&self) -> Result<Vec<u8>, Box<dyn Error>> {
    let mut buffer = Vec::<u8>::new();

    // https://docin.divoom-gz.com/web/#/5/247
    let mut writer = BufWriter::new(&mut buffer);
    // Indicates which alarm to set, starting from 0.
    writer.write_u8(self.index)?;
    // 1 (alarm on), 0 (alarm off).
    writer.write_u8(self.enable as u8)?;
    // Hour to set for the alarm.
    writer.write_u8(self.time.hour() as u8)?;
    // Minute to set for the alarm.
    writer.write_u8(self.time.minute() as u8)?;
    // Bits 0 to 6 represent Sunday to Saturday, respectively. Set to 1 if the alarm should repeat on that day.
    writer.write_u8(self.repeat)?;
    // Alarm mode (ALARM_MUSIC=0, and others: 1, 2, 3, 4).
    writer.write_u8(self.mode)?;
    // Alarm trigger mode (ALARM_TRIGGER_MUSIC=1, ALARM_TRIGGER_GIF=4).
    writer.write_u8(self.trigger_mode)?;
    // If the trigger mode is ALARM_TRIGGER_MUSIC, these 2 bytes represent the frequency point.
    writer.write_all(&self.fm)?;
    // Volume level for the alarm, ranging from 0 to 100.
    writer.write_u8(self.volume)?;
    drop(writer);

    Ok(buffer)
  }
}
