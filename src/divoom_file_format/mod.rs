use std::error::Error;
use std::fs::File;
use std::io::BufReader;

use image::codecs::gif::GifDecoder;
use image::{AnimationDecoder, DynamicImage, GenericImageView, Rgb};
use indexmap::IndexSet;

pub mod animation;
pub mod frame;
pub mod frame_header;

use animation::Animation;
use frame::Frame;
use frame_header::FrameHeader;

pub fn read_divoom_16x16_animation_from_file(filename: &str) -> Result<Animation, Box<dyn Error>> {
  File::open(filename)
    .map(BufReader::new)
    .map_err(|e| e.into())
    .and_then(|mut reader| Animation::from_16x16(&mut reader))
}

fn get_palette_from_images(images: &[DynamicImage]) -> IndexSet<Rgb<u8>> {
  images
    .iter()
    .flat_map(|image| {
      image
        .pixels()
        .map(|(_x, _y, pixel_data)| Rgb([pixel_data[0], pixel_data[1], pixel_data[2]]))
        .collect::<IndexSet<_>>()
    })
    .collect()
}

pub fn read_gif_from_file(filename: &str) -> Result<Animation, Box<dyn Error>> {
  let file = File::open(filename)?;
  let reader = BufReader::new(file);
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
