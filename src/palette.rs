use sdl2::pixels::Color;

pub fn hex_to_color(hex: u32) -> Color {
  Color::RGB((hex >> 16) as u8, (hex >> 8) as u8, hex as u8)
}

pub fn pear_36() -> Vec<Color> {
  vec![
    hex_to_color(0x5e315b),
    hex_to_color(0x8c3f5d),
    hex_to_color(0xba6156),
    hex_to_color(0xf2a65e),
    hex_to_color(0xffe478),
    hex_to_color(0xcfff70),
    hex_to_color(0x8fde5d),
    hex_to_color(0x3ca370),
    hex_to_color(0x3d6e70),
    hex_to_color(0x323e4f),
    hex_to_color(0x322947),
    hex_to_color(0x473b78),
    hex_to_color(0x4b5bab),
    hex_to_color(0x4da6ff),
    hex_to_color(0x66ffe3),
    hex_to_color(0xffffeb),
    hex_to_color(0xc2c2d1),
    hex_to_color(0x7e7e8f),
    hex_to_color(0x606070),
    hex_to_color(0x43434f),
    hex_to_color(0x272736),
    hex_to_color(0x3e2347),
    hex_to_color(0x57294b),
    hex_to_color(0x964253),
    hex_to_color(0xe36956),
    hex_to_color(0xffb570),
    hex_to_color(0xff9166),
    hex_to_color(0xeb564b),
    hex_to_color(0xb0305c),
    hex_to_color(0x73275c),
    hex_to_color(0x422445),
    hex_to_color(0x5a265e),
    hex_to_color(0x80366b),
    hex_to_color(0xbd4882),
    hex_to_color(0xff6b97),
    hex_to_color(0xffb5b5),
  ]
}

pub fn sweetie_16() -> Vec<u32> {
  vec![
    0x1a1c2cff, 0x5d275dff, 0xb13e53ff, 0xef7d57ff, 0xffcd75ff, 0xa7f070ff, 0x38b764ff,
    0x257179ff, 0x29366fff, 0x3b5dc9ff, 0x41a6f6ff, 0x73eff7ff, 0xf4f4f4ff, 0x94b0c2ff,
    0x566c86ff, 0x333c57ff,
  ]
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Sweetie16 {
  Black = 0,
  Purple = 1,
  Red = 2,
  Orange = 3,
  Yellow = 4,
  LightGreen = 5,
  DarkGreen = 6,
  Teal = 7,
  DeepBlue = 8,
  DarkBlue = 9,
  LightBlue = 10,
  Aqua = 11,
  White = 12,
  LightGray = 13,
  MediumGray = 14,
  DarkGray = 15,
}

impl From<Sweetie16> for u8 {
  fn from(color: Sweetie16) -> Self {
    color as u8
  }
}
