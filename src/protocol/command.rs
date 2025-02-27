#[derive(Clone, Copy, Debug)]
pub enum Command {
  Alarm,
  Animation
}

impl Command {
  pub fn value(&self) -> u8 {
    match *self {
      Command::Alarm => 0x43,
      Command::Animation => 0x8b
    }
  }
}
