use std::error::Error;
use std::io::{Read, Write};
use std::time::Duration;

use image::codecs::gif::{GifEncoder, Repeat};
use image::Delay;

use super::frame::Frame;

#[derive(Debug)]
pub struct Animation {
  pub frames: Vec<Frame>
}

impl Animation {
  pub fn from_16x16<R: Read>(reader: &mut R) -> Result<Animation, Box<dyn Error>> {
    let mut frames = Vec::new();

    loop {
      match Frame::from_16x16(reader, frames.last().map_or(&[], |f: &Frame| &f.palette)) {
        Ok(frame) => frames.push(frame),
        Err(e) => {
          if let Some(io_error) = e.downcast_ref::<std::io::Error>() {
            // Unfortunately, I don't know an easier way to catch an EOF error
            // This is not an error, but just the end of the file, so return what we've got so far.
            if io_error.kind() == std::io::ErrorKind::UnexpectedEof {
              break;
            }
          } else {
            return Err(e);
          }
        }
      }
    }

    Ok(Animation { frames })
  }

  pub fn save_to_gif<W: Write>(&self, writer: &mut W) -> Result<(), Box<dyn Error>> {
    let mut encoder = GifEncoder::new(writer);
    encoder.set_repeat(Repeat::Infinite)?;

    self
      .frames
      .iter()
      .try_for_each(|image| -> Result<(), Box<dyn Error>> {
        let frame = image::Frame::from_parts(
          image.image.clone().into(),
          0,
          0,
          Delay::from_saturating_duration(Duration::from_millis(
            image.header.time_in_milliseconds.into()
          ))
        );

        Ok(encoder.encode_frame(frame)?)
      })
  }

  pub fn save_to_divoom_format<W: Write>(&self, mut writer: &mut W) -> Result<(), Box<dyn Error>> {
    let mut palette = Vec::new();

    self
      .frames
      .iter()
      .try_for_each(|frame| -> Result<(), Box<dyn Error>> {
        if frame.header.reuse_palette {
          // TODO: I think the palette is not correct here.
          // frame.palette exists of all colors in that frame,
          // and palette might already contain some of that colors.
          // I think the best solution is to only store new colors in case of reuse-palette=true,
          // or to have a field local_palette in Frame.
          // Needs a test.
          palette.extend(frame.palette.clone());
        } else {
          palette = frame.palette.clone();
        }
        frame.serialize(&palette, &mut writer)?;
        Ok(())
      })
  }
}
