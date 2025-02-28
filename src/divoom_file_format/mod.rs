use std::error::Error;
use std::fs::File;
use std::io::{BufReader, BufWriter, Read};
use std::time::Duration;

use bitstream_io::{BitRead, BitReader};
use byteorder::ReadBytesExt;
use image::{Delay, DynamicImage, Rgb, RgbImage};
use image::codecs::gif::GifEncoder;
use log::{debug, info};

pub mod frame_header;

use frame_header::FrameHeader;

#[derive(Debug)]
pub struct Frame {
  pub header: FrameHeader,
  pub palette: Vec<Rgb<u8>>,
  pub image: DynamicImage
}

fn read_divoom_16x16_frame<R: Read>(
  reader: &mut R,
  previous_palette: &[Rgb<u8>]
) -> Result<Frame, Box<dyn Error>> {
  let mut image = RgbImage::new(16, 16);

  let header = FrameHeader::from_reader(reader)?;
  info!("Image frame header: {:?}", header);

  if header.magic_number != 0xAA {
    return Err(
      format!(
        "Magic number does not match {:#04X}: {:#04X}",
        0xAA, header.magic_number
      )
      .into()
    );
  }

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

fn read_divoom_16x16_animation<R: Read>(reader: &mut R) -> Result<Vec<Frame>, Box<dyn Error>> {
  let mut frames = Vec::new();

  loop {
    match read_divoom_16x16_frame(
      reader,
      frames.last().map_or(&[], |f: &Frame| &f.palette)
    ) {
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

  Ok(frames)
}

pub fn read_divoom_16x16_animation_from_file(filename: String) -> Result<Vec<Frame>, Box<dyn Error>> {
  let file = File::open(filename)?;
  let mut reader = BufReader::new(file);
  read_divoom_16x16_animation(&mut reader)
}

pub fn save_animation_to_gif(frames: &[Frame], filename: &str) -> Result<(), Box<dyn Error>> {
  let file = File::create(filename)?;
  let writer = BufWriter::new(file);
  let mut encoder = GifEncoder::new(writer);
  encoder.set_repeat(image::codecs::gif::Repeat::Infinite);

  frames.iter().try_for_each(|image| -> Result<(), Box<dyn Error>> {
    let frame = image::Frame::from_parts(image.image.clone().into(), 0, 0,
      Delay::from_saturating_duration(Duration::from_millis(image.header.time_in_milliseconds.into())));

    Ok(encoder.encode_frame(frame)?)
  })
}
