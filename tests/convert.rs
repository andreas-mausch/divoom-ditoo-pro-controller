use std::error::Error;
use std::fs::{read, File};
use std::io::{BufReader, BufWriter};

use spectral::prelude::*;

use divoom_ditoo_pro_controller::divoom_file_format::animation::Animation;

#[test]
fn convert_divoom16_to_gif() -> Result<(), Box<dyn Error>> {
  let animation = Animation::from_16x16(&mut BufReader::new(File::open("./images/witch.divoom16")?))?;

  let mut output = Vec::new();
  animation.save_to_gif(&mut BufWriter::new(&mut output))?;

  let expected_output = read("./images/witch.gif")?;
  asserting("GIF output").that(&output).is_equal_to(&expected_output);

  Ok(())
}
