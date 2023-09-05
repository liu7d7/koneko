use std::cmp::min;
use std::collections::HashMap;
use std::fmt::Debug;
use std::time::SystemTime;

use image::GenericImageView;
use image::io::Reader as ImageReader;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;

use crate::csv;
use crate::lex_parse_basic::{BASIC, ParseOptions, Token};
use crate::palette::Sweetie16;

pub(crate) const WIDTH: i32 = 480;
pub(crate) const HEIGHT: i32 = 300;
pub(crate) const TEXT_HEIGHT: i32 = HEIGHT / 12;
pub(crate) const FONT_TEXTURE_SIZE: i32 = 160;
pub(crate) static mut PROGRAM_BEGIN: u128 = 0;
pub(crate) const COLOR_PREFIX: u8 = b'`';

#[inline]
pub fn millis() -> u128 {
  SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_millis()
}

pub fn secs_since_start() -> f64 {
  (millis() - unsafe { PROGRAM_BEGIN }) as f64 / 1000.0
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Character {
  pub top_left_x: i32,
  pub top_left_y: i32,
  pub bottom_right_x: i32,
  pub bottom_right_y: i32,
  pub char: u8,
}

impl Character {
  pub fn invalid() -> Character {
    Character {
      top_left_x: 0,
      top_left_y: 0,
      bottom_right_x: 0,
      bottom_right_y: 0,
      char: 0,
    }
  }
}

pub(crate) const BASIC_SCREEN: i32 = 0;
pub(crate) const EXEC_SCREEN: i32 = 1;

pub struct Koneko {
  pub palette: Vec<u32>,
  pub video: Box<[u32; WIDTH as usize * HEIGHT as usize]>,
  pub basic: BASIC,
  pub char_info: [Character; 256],
  pub font: [[bool; FONT_TEXTURE_SIZE as usize]; FONT_TEXTURE_SIZE as usize],
  pub screen: i32,
  pub printed_text: Vec<String>,
  pub current_line: String,
  pub current_line_highlighted: String,
  pub line_cursor: i32,
  pub cursor: i32,
  pub scroll: i32,
  pub prev_cursor_on: bool,
  pub error: Option<String>,
  pub ok: Option<String>,
}

impl Koneko {
  pub fn new(palette: Vec<u32>, font_path: &str) -> Koneko {
    unsafe { PROGRAM_BEGIN = millis(); }
    let mut char_info = [Character::invalid(); 256];
    let font_csv = csv::read_csv((String::from(font_path) + ".config.csv").as_str());
    for row in font_csv {
      if row.len() != 5 {
        panic!("Invalid font.csv")
      }

      let char: u8 = if let csv::Value::Int(code) = &row[0] {
        *code as u8
      } else {
        panic!("Invalid font.csv")
      };

      let top_left_x: i32 = if let csv::Value::Int(x) = &row[1] {
        *x as i32
      } else {
        panic!("Invalid font.csv")
      };

      let top_left_y: i32 = if let csv::Value::Int(y) = &row[2] {
        *y as i32
      } else {
        panic!("Invalid font.csv")
      };

      let bottom_right_x: i32 = if let csv::Value::Int(x) = &row[3] {
        *x as i32
      } else {
        panic!("Invalid font.csv")
      };

      let bottom_right_y: i32 = if let csv::Value::Int(y) = &row[4] {
        *y as i32
      } else {
        panic!("Invalid font.csv")
      };

      char_info[char as usize] = Character {
        top_left_x,
        top_left_y,
        bottom_right_x,
        bottom_right_y,
        char,
      };
    }

    let font_pixels = {
      // pixels with r, g, b = 255, 255, 255 are true, else false
      let mut font_pixels = [[false; FONT_TEXTURE_SIZE as usize]; FONT_TEXTURE_SIZE as usize];
      let font_image = ImageReader::open("font.png").unwrap().decode().unwrap();
      if font_image.dimensions() != (160, 160) {
        panic!("Invalid font.png. Expected image of 160x160 pixels, got {:?}", font_image.dimensions());
      }

      for i in 0..160 {
        for j in 0..160 {
          let pixel = font_image.get_pixel(i, j);
          if pixel[0] == 255 && pixel[1] == 255 && pixel[2] == 255 {
            font_pixels[i as usize][j as usize] = true;
          }
        }
      }

      font_pixels
    };

    let symbols = HashMap::from([
      (b'(', Token::LParen),
      (b')', Token::RParen),
      (b'[', Token::LSquare),
      (b']', Token::RSquare),
      (b'{', Token::LCurly),
      (b'}', Token::RCurly),
      (b'+', Token::Add),
      (b'-', Token::Sub),
      (b'*', Token::Mul),
      (b'/', Token::Div),
      (b'|', Token::Pipe),
      (b'&', Token::Ampersand),
      (b'!', Token::Exclamation),
      (b'%', Token::Percent),
      (b',', Token::Comma),
    ]);

    let keywords = HashMap::from([
      ("to", Token::To),
      ("step", Token::Step),
      ("then", Token::Then),
      ("else", Token::Else)
    ]);

    let options = ParseOptions {
      builtin_commands: vec![
        "print",
        "next",
        "loop",
        "while",
        "sin",
        "cos",
        "goto",
        "gosub",
        "end",
        "ret",
        "dot",
        "time",
        "cls",
        "delay",
        "refresh",
        "poly",
        "line",
        "str",
        "int",
        "chr",
        "rnd",
        "rad",
        "deg",
        "save",
        "load",
        "new",
        "rim",
        "text",
      ]
    };

    let mut ko = Koneko {
      palette,
      video: Box::new([0; WIDTH as usize * HEIGHT as usize]),
      basic: BASIC::new(symbols, keywords, options),
      char_info,
      font: font_pixels,
      screen: BASIC_SCREEN,
      printed_text: vec![],
      current_line: "".to_string(),
      current_line_highlighted: "".to_string(),
      line_cursor: 0,
      cursor: 0,
      scroll: 0,
      prev_cursor_on: false,
      error: None,
      ok: None,
    };

    ko.redraw_screen();
    ko
  }

  pub fn on_key(&mut self, event: Event) {
    match event {
      Event::KeyDown { keycode, keymod, .. } => {
        match keycode {
          Some(Keycode::Tab) => {
            let line_empty = self.current_line.is_empty();
            let line_cursor_valid = self.line_cursor < self.basic.program.len() as i32;
            if keymod.contains(sdl2::keyboard::Mod::LCTRLMOD) {
              self.screen = (self.screen + 1) % 2;
              self.redraw_screen();
            } else if line_empty && line_cursor_valid {
              self.current_line = self.basic.program[self.line_cursor as usize].contents.clone();
              self.cursor = self.current_line.len() as i32;
            }
          }
          Some(Keycode::Backspace) => {
            if self.screen == BASIC_SCREEN {
              if self.cursor > 0 {
                self.current_line.remove(self.cursor as usize - 1);
                self.cursor -= 1;
              }
            }
          }
          Some(Keycode::Delete) => {
            if self.screen == BASIC_SCREEN {
              if self.cursor < self.current_line.len() as i32 {
                self.current_line.remove(self.cursor as usize);
              }
            }
          }
          Some(Keycode::Home) => {
            if self.screen == BASIC_SCREEN {
              self.cursor = 0;
            }
          }
          Some(Keycode::End) => {
            if self.screen == BASIC_SCREEN {
              self.cursor = self.current_line.len() as i32;
            }
          }
          Some(Keycode::Left) => {
            if self.screen == BASIC_SCREEN {
              if self.cursor > 0 {
                self.cursor -= 1;
              }
            }
          }
          Some(Keycode::Right) => {
            if self.screen == BASIC_SCREEN {
              if self.cursor < self.current_line.len() as i32 {
                self.cursor += 1;
              }
            }
          }
          Some(Keycode::Up) => {
            if self.screen == BASIC_SCREEN && self.line_cursor > 0 {
              self.line_cursor -= 1;
              self.redraw_screen();
            }
          }
          Some(Keycode::Down) => {
            let can_go_down = self.line_cursor < self.basic.program.len() as i32 - 1;
            if self.screen == BASIC_SCREEN && can_go_down {
              self.line_cursor += 1;
              self.redraw_screen();
            }
          }
          Some(Keycode::Return) => {
            if self.screen == BASIC_SCREEN {
              self.error = None;
              self.ok = None;
              let res = self.basic.add_line(self.current_line.clone());
              if let Err(error) = res {
                self.error = Some(error);
              } else if let Ok(Some(node)) = res {
                let res = self.interpret(node);
                if let Err(error) = res {
                  self.error = Some(error);
                } else if let Ok(value) = res {
                  self.ok = Some(self.current_line.clone() + " -> " + value.to_string(true).as_str());
                }
                self.current_line.clear();
                self.cursor = 0;
              } else {
                self.error = None;
                self.current_line.clear();
                self.cursor = 0;
              }
              self.redraw_screen();
            }
          }
          _ => {}
        }
      }
      _ => {}
    }

    if self.screen == BASIC_SCREEN {
      self.current_line_highlighted = self.highlight_string(self.current_line.clone());
    }
  }

  pub fn on_text_input(&mut self, event: Event) {
    if let Event::TextInput { text, .. } = event {
      if self.screen == BASIC_SCREEN {
        self.current_line.insert_str(self.cursor as usize, text.as_str());
        self.current_line_highlighted = self.highlight_string(self.current_line.clone());
        self.cursor += text.len() as i32;
      }
    }
  }

  fn color_for_token(&self, token: &Token) -> Sweetie16 {
    match token {
      Token::To | Token::Step | Token::Then | Token::Else => Sweetie16::Pink,
      Token::Integer(_) | Token::Float(_) => Sweetie16::Orange,
      Token::String(_) => Sweetie16::LightGreen,
      Token::Identifier(id) => {
        if self.basic.is_builtin_command(id.as_str()) || id == "for" || id == "if" {
          Sweetie16::Pink
        } else {
          Sweetie16::Yellow
        }
      }
      _ => Sweetie16::Aqua
    }
  }

  fn one_hex_digit_to_char(it: u8) -> Option<u8> {
    match it {
      0..=9 => Some(it + b'0'),
      10..=15 => Some(it - 10 + b'a'),
      _ => None,
    }
  }

  pub fn highlight_string(&self, mut str: String) -> String {
    let (tokens, _err) = self.basic.lex_line(&str);
    let mut inserted = 0;
    for (token, begin, end) in tokens {
      let mut color_string = "` ".to_string();
      color_string.replace_range(1..=1, String::from(Self::one_hex_digit_to_char(self.color_for_token(&token) as u8).unwrap() as char).as_str());

      str.insert_str(begin + inserted, color_string.as_str());
      inserted += 2;

      str.insert_str(end + inserted, "`r");
      inserted += 2;
    }

    str
  }

  fn redraw_screen(&mut self) {
    self.printed_text.clear();
    match self.screen {
      BASIC_SCREEN => {
        self.cls(Sweetie16::Black);
        for i in self.basic.program.len() - min(TEXT_HEIGHT as usize - 1, self.basic.program.len())..self.basic.program.len() {
          let mut display = self.highlight_string(self.basic.program[i].contents.clone());

          if i == self.line_cursor as usize {
            display = String::from("> ") + display.as_str();
          }

          self.text(display.as_str(), 3, 3 + i as i32 * 12, Sweetie16::White, None::<u8>, None::<u8>)
        }

        if let Some(error) = &self.error {
          self.text(error.clone().as_str(), 3, HEIGHT - 27, Sweetie16::Red, None::<u8>, None::<u8>)
        } else if let Some(ok) = &self.ok {
          self.text(ok.clone().as_str(), 3, HEIGHT - 27, Sweetie16::LightGreen, None::<u8>, None::<u8>)
        }
      }
      EXEC_SCREEN => {
        self.basic.reset_program_state();
        self.cls(Sweetie16::Black);
      }
      _ => panic!("Unknown screen {}", self.screen)
    }
  }

  pub(crate) fn draw_screen(&mut self) {
    match self.screen {
      BASIC_SCREEN => {
        self.rect(0, HEIGHT - 15, WIDTH, 15, Sweetie16::Black);
        self.text(
          ("basic: ".to_string() + self.current_line_highlighted.as_str()).as_str(),
          3,
          HEIGHT - 13,
          Sweetie16::White,
          None::<u8>,
          None::<u8>,
        );

        let cursor_on = millis() % 1000 < 500;
        if cursor_on {
          self.text(
            "       _",
            3 + self.width(self.current_line[0..self.cursor as usize].to_string().as_str()),
            HEIGHT - 12,
            Sweetie16::White,
            None::<u8>,
            None::<u8>,
          );
        }
      }
      EXEC_SCREEN => {}
      _ => panic!("Unknown screen {}", self.screen)
    }
  }

  pub fn execute_code(&mut self) -> Result<(), String> {
    if self.screen == EXEC_SCREEN {
      let begin = millis();
      while !self.basic.refresh && millis() - begin < 1000 {
        if self.basic.line_no >= self.basic.program.len() {
          break;
        }
        self.exec_current_line()?;
      }

      if millis() - begin >= 1000 {
        self.screen = BASIC_SCREEN;
        self.error = Some("Timeout, try adding a refresh statement".to_string());
        self.redraw_screen();
      }

      self.basic.refresh = false;
    }
    Ok(())
  }

  pub fn print(&mut self, text: String) {
    if self.printed_text.len() + 1 > TEXT_HEIGHT as usize {
      // redraw whole text screen
      self.printed_text.remove(0);
      self.printed_text.push(String::from(text));

      for i in 0..TEXT_HEIGHT {
        self.text(
          self.printed_text[i as usize].clone().as_str(),
          2,
          i * 12 + 2,
          Sweetie16::White,
          Some(Sweetie16::DarkGray),
          Some(Sweetie16::Black),
        );
      }
    } else {
      self.printed_text.push(String::from(text));
      self.text(
        self.printed_text[self.printed_text.len() - 1].clone().as_str(),
        2,
        (self.printed_text.len() - 1) as i32 * 12 + 2,
        Sweetie16::White,
        Some(Sweetie16::DarkGray),
        Some(Sweetie16::Black),
      );
    }
  }
}