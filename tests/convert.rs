use std::error::Error;
use std::fs::{read, File};
use std::io::{BufReader, BufWriter};

use spectral::prelude::*;

use divoom_ditoo_pro_controller::divoom_file_format::animation::Animation;

#[test]
fn convert_divoom16_to_gif() -> Result<(), Box<dyn Error>> {
  let animation =
    Animation::from_16x16(&mut BufReader::new(File::open("./images/witch.divoom16")?))?;

  let mut output = Vec::new();
  animation.save_to_gif(&mut BufWriter::new(&mut output))?;

  let expected_output = read("./images/witch.gif")?;
  asserting("GIF output")
    .that(&output)
    .is_equal_to(&expected_output);

  Ok(())
}

#[test]
fn convert_gif_to_divoom16() -> Result<(), Box<dyn Error>> {
  let animation = Animation::from_gif(&mut BufReader::new(File::open("./images/witch.gif")?))?;

  let mut output = Vec::new();
  animation.save_to_divoom_format(&mut BufWriter::new(&mut output))?;

  // The delays are different when converting back to divoom16:
  // .gif stores the delay as centiseconds,
  // .divoom16 as milliseconds.
  // Therefore, the 127ms are converted to 120ms for the .gif.
  asserting("Divoom16 delay")
    .that(&output[3])
    .is_equal_to(120);
  asserting("Divoom16 delay")
    .that(&output[130])
    .is_equal_to(120);
  asserting("Divoom16 delay")
    .that(&output[233])
    .is_equal_to(120);
  asserting("Divoom16 delay")
    .that(&output[336])
    .is_equal_to(120);

  // Overwrite back the original 127ms, so the buffer should be completely identical again.
  output[3] = 127;
  output[130] = 127;
  output[233] = 127;
  output[336] = 127;

  let expected_output = read("./images/witch.divoom16")?;
  asserting("Divoom16 output")
    .that(&output)
    .is_equal_to(&expected_output);

  Ok(())
}
