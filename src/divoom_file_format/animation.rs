use std::error::Error;
use std::io::{BufRead, Read, Seek, Write};
use std::time::Duration;

use image::codecs::gif::{GifDecoder, GifEncoder, Repeat};
use image::{AnimationDecoder, DynamicImage, Delay};

use super::get_palette_from_images;
use super::frame::Frame;
use super::frame_header::FrameHeader;

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

  pub fn from_gif<R: BufRead + Read + Seek>(reader: &mut R) -> Result<Animation, Box<dyn Error>> {
    let decoder = GifDecoder::new(reader)?;
    let frames = decoder.into_frames().collect_frames()?;
    let palette = get_palette_from_images(
      &frames
        .iter()
        .map(|frame| frame.buffer().clone().into())
        .collect::<Vec<_>>()
    )
    .into_iter()
    .collect::<Vec<_>>();

    if palette.len() >= 256 {
      return Err(
        format!(
          "Too many colors in the .gif, a maximum of {} is supported, but {} found",
          256,
          palette.len()
        )
        .into()
      );
    }

    Ok(Animation {
      frames: frames
        .iter()
        .enumerate()
        .map(|(index, frame)| {
          let numer_denom_ms = frame.delay().numer_denom_ms();
          let time_in_milliseconds = (numer_denom_ms.0 as f64 / numer_denom_ms.1 as f64) as u16;
          let header = if index == 0 {
            FrameHeader {
              time_in_milliseconds,
              reuse_palette: false,
              color_count: palette.len() as u8
            }
          } else {
            FrameHeader {
              time_in_milliseconds,
              reuse_palette: true,
              color_count: 0
            }
          };

          Frame {
            header,
            palette: palette.clone(),
            local_palette: if index == 0 {
              palette.clone()
            } else {
              Vec::new()
            },
            image: DynamicImage::from(frame.buffer().clone())
          }
        })
        .collect::<Vec<_>>()
    })
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
          palette.extend(frame.local_palette.clone());
        } else {
          palette = frame.palette.clone();
        }
        frame.serialize(&palette, &mut writer)?;
        Ok(())
      })
  }
}
