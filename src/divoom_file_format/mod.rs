use std::error::Error;
use std::fs::File;
use std::io::BufReader;

use image::{DynamicImage, GenericImageView, Rgb};
use indexmap::IndexSet;

pub mod animation;
pub mod frame;
pub mod frame_header;

use animation::Animation;

pub fn read_divoom_16x16_animation_from_file(filename: &str) -> Result<Animation, Box<dyn Error>> {
  File::open(filename)
    .map(BufReader::new)
    .map_err(|e| e.into())
    .and_then(|mut reader| Animation::from_16x16(&mut reader))
}

fn _get_palette_from_images(images: &[DynamicImage]) -> IndexSet<Rgb<u8>> {
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
