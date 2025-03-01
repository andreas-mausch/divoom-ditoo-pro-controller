use std::error::Error;
use std::io::Read;

use bitstream_io::{BitRead, BitReader};
use byteorder::ReadBytesExt;
use image::{DynamicImage, Rgb, RgbImage};
use log::{debug, info};

use super::frame_header::FrameHeader;

#[derive(Debug)]
pub struct Frame {
  pub header: FrameHeader,
  pub palette: Vec<Rgb<u8>>,
  pub image: DynamicImage
}

impl Frame {
  pub fn from_16x16<R: Read>(
    reader: &mut R,
    previous_palette: &[Rgb<u8>]
  ) -> Result<Frame, Box<dyn Error>> {
    let mut image = RgbImage::new(16, 16);

    let header = FrameHeader::from_reader(reader)?;
    info!("Image frame header: {:?}", header);

    let mut palette = if header.reuse_palette {
      previous_palette.to_vec()
    } else {
      Vec::new()
    };

    for _ in 0..header.color_count {
      let red = reader.read_u8()?;
      let green = reader.read_u8()?;
      let blue = reader.read_u8()?;

      info!(
        "Adding color to palette: #{:02X}{:02X}{:02X}",
        red, green, blue
      );
      palette.push(Rgb([red, green, blue]));
    }

    let bits_per_pixel: u8 = f32::log2(palette.len() as f32).ceil() as u8;
    info!(
      "Color count: {}; Bits per pixel: {}",
      palette.len(),
      bits_per_pixel
    );

    let width = 16u32;
    let height = 16u32;
    let pixel_data_in_bits = width * height * bits_per_pixel as u32;
    let pixel_data_in_bytes = pixel_data_in_bits.div_ceil(8);
    info!(
      "Pixel data is: {} bits = {} bytes",
      pixel_data_in_bits, pixel_data_in_bytes
    );

    let mut pixel_data_reader = BitReader::endian(
      reader.take(pixel_data_in_bytes.into()),
      bitstream_io::LittleEndian
    );

    for y in 0..height {
      for x in 0..width {
        let palette_index = pixel_data_reader.read::<u8>(bits_per_pixel.into())?;
        let palette_entry = palette[palette_index as usize];
        debug!(
          "Palette index {}x{}: {} ({:?})",
          x, y, palette_index, palette_entry
        );

        image.put_pixel(x, y, palette_entry);
      }
    }

    Ok(Frame {
      header,
      palette,
      image: DynamicImage::ImageRgb8(image)
    })
  }
}
