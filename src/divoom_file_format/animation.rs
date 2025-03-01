use std::error::Error;
use std::fs::File;
use std::io::{Read, BufWriter};
use std::time::Duration;

use image::codecs::gif::{GifEncoder, Repeat};
use image::Delay;

use super::frame::Frame;

#[derive(Debug)]
pub struct Animation {
  frames: Vec<Frame>
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

  pub fn save_to_gif(&self, filename: &str) -> Result<(), Box<dyn Error>> {
    let file = File::create(filename)?;
    let writer = BufWriter::new(file);
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
}
