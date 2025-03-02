use std::error::Error;
use std::fs::read;
use std::io::BufWriter;

use spectral::prelude::*;

use divoom_ditoo_pro_controller::divoom_file_format::read_divoom_16x16_animation_from_file;

#[test]
fn convert_divoom16_to_gif() -> Result<(), Box<dyn Error>> {
  let animation = read_divoom_16x16_animation_from_file("./images/witch.divoom16")?;

  let mut output = Vec::new();
  animation.save_to_gif(&mut BufWriter::new(&mut output))?;

  let expected_output = read("./images/witch.gif")?;
  asserting("GIF output").that(&output).is_equal_to(&expected_output);

  Ok(())
}
