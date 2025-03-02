use std::error::Error;
use std::io::{Read, Write};

use bitstream_io::{BitRead, BitReader, BitWrite, BitWriter};
use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use image::{DynamicImage, GenericImageView, Pixel, Rgb, RgbImage};
use log::{debug, info};

use super::frame_header::{FrameHeader, FRAME_HEADER_MAGIC_NUMBER};

#[derive(Debug)]
pub struct Frame {
  pub header: FrameHeader,
  pub palette: Vec<Rgb<u8>>,
  pub local_palette: Vec<Rgb<u8>>,
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

    let previous_palette = if header.reuse_palette {
      previous_palette.to_vec()
    } else {
      Vec::new()
    };

    let mut local_palette = Vec::new();

    for _ in 0..header.color_count {
      let red = reader.read_u8()?;
      let green = reader.read_u8()?;
      let blue = reader.read_u8()?;

      info!(
        "Adding color to palette: #{:02X}{:02X}{:02X}",
        red, green, blue
      );
      local_palette.push(Rgb([red, green, blue]));
    }

    let palette = [previous_palette.as_slice(), local_palette.as_slice()].concat();

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
      local_palette,
      image: DynamicImage::ImageRgb8(image)
    })
  }

  pub fn serialize<W: Write>(
    &self,
    palette: &[Rgb<u8>],
    writer: &mut W
  ) -> Result<(), Box<dyn Error>> {
    let pixel_data = self.build_pixel_data(palette)?;
    let length = 7 + self.palette.len() * 3 + pixel_data.len();

    writer.write_u8(FRAME_HEADER_MAGIC_NUMBER)?;
    writer.write_u16::<LittleEndian>(length as u16)?;
    writer.write_u16::<LittleEndian>(self.header.time_in_milliseconds)?;
    writer.write_u8(self.header.reuse_palette as u8)?;
    writer.write_u8(self.header.color_count)?;

    self
      .palette
      .iter()
      .try_for_each(|color| writer.write_all(color.channels()))?;

    writer.write_all(&pixel_data)?;
    Ok(())
  }

  fn build_pixel_data(&self, palette: &[Rgb<u8>]) -> Result<Vec<u8>, Box<dyn Error>> {
    let bits_per_pixel: u8 = f32::log2(palette.len() as f32).ceil() as u8;
    let width = 16;
    let height = 16;

    let mut pixel_data = Vec::new();
    let mut pixel_data_writer = BitWriter::endian(&mut pixel_data, bitstream_io::LittleEndian);

    for y in 0..height {
      for x in 0..width {
        let pixel = self.image.get_pixel(x, y).to_rgb();
        let palette_index = palette
          .iter()
          .position(|&color| color == pixel)
          .ok_or("Pixel not found in palette")? as u8;
        pixel_data_writer.write::<u8>(bits_per_pixel.into(), palette_index)?;
      }
    }

    Ok(pixel_data)
  }
}
