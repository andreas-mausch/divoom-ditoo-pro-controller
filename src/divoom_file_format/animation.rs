use std::error::Error;
use std::fs::File;
use std::io::BufWriter;
use std::time::Duration;

use image::codecs::gif::{GifEncoder, Repeat};
use image::Delay;

use super::Frame;

#[derive(Debug)]
pub struct Animation {
  frames: Vec<Frame>
}

impl Animation {
  pub fn from(frames: Vec<Frame>) -> Animation {
    Animation { frames }
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
