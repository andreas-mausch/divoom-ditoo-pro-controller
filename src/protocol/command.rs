#[derive(Clone, Copy, Debug)]
pub enum Command {
  Alarm,
  Animation,
  GetSettings,
  SetChannel,
  SetDateTime
}

impl Command {
  pub fn value(&self) -> u8 {
    match *self {
      Command::SetDateTime => 0x18,
      Command::Alarm => 0x43,
      Command::SetChannel => 0x45,
      Command::GetSettings => 0x46,
      Command::Animation => 0x8b
    }
  }
}
