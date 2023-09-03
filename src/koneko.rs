use std::cmp::{max, min};
use std::collections::HashMap;
use std::fmt::Debug;
use std::fs::File;
use std::io::{Read, Write};
use std::time::SystemTime;

use image::GenericImageView;
use image::io::Reader as ImageReader;
use rand::Rng;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;

use crate::basic::{BASIC, Node, ParseOptions, Token, Value};
use crate::csv;
use crate::palette::Sweetie16;

pub(crate) const WIDTH: u32 = 480;
pub(crate) const HEIGHT: u32 = 300;
pub(crate) const TEXT_WIDTH: u32 = WIDTH / 5 - 1;
pub(crate) const TEXT_HEIGHT: u32 = HEIGHT / 12;
pub(crate) const FONT_TEXTURE_SIZE: u32 = 160;
pub(crate) static mut PROGRAM_BEGIN: u128 = 0;

pub fn secs_since_start() -> f64 {
  (SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_millis() - unsafe { PROGRAM_BEGIN }) as f64 / 1000.0
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Character {
  pub top_left_x: u32,
  pub top_left_y: u32,
  pub bottom_right_x: u32,
  pub bottom_right_y: u32,
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

pub(crate) const BASIC_SCREEN: u32 = 0;
pub(crate) const EXEC_SCREEN: u32 = 1;

pub struct Koneko {
  pub palette: Vec<u32>,
  pub video: Box<[u32; WIDTH as usize * HEIGHT as usize]>,
  pub basic: BASIC,
  pub char_info: [Character; 256],
  pub font_pixels: [[bool; FONT_TEXTURE_SIZE as usize]; FONT_TEXTURE_SIZE as usize],
  pub screen: u32,
  pub printed_text: Vec<String>,
  pub current_line: String,
  pub line_cursor: u32,
  pub cursor: u32,
  pub scroll: i32,
  pub prev_cursor_on: bool,
  pub error: Option<String>,
}

impl Koneko {
  pub fn new(palette: Vec<u32>, font_path: &str) -> Koneko {
    unsafe { PROGRAM_BEGIN = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_millis(); }
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

      let top_left_x: u32 = if let csv::Value::Int(x) = &row[1] {
        *x as u32
      } else {
        panic!("Invalid font.csv")
      };

      let top_left_y: u32 = if let csv::Value::Int(y) = &row[2] {
        *y as u32
      } else {
        panic!("Invalid font.csv")
      };

      let bottom_right_x: u32 = if let csv::Value::Int(x) = &row[3] {
        *x as u32
      } else {
        panic!("Invalid font.csv")
      };

      let bottom_right_y: u32 = if let csv::Value::Int(y) = &row[4] {
        *y as u32
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
      ("step", Token::Step)
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
        "load"
      ]
    };

    let mut ko = Koneko {
      palette,
      video: Box::new([0; WIDTH as usize * HEIGHT as usize]),
      basic: BASIC::new(symbols, keywords, options),
      char_info,
      font_pixels,
      screen: BASIC_SCREEN,
      printed_text: vec![],
      current_line: "".to_string(),
      line_cursor: 0,
      cursor: 0,
      scroll: 0,
      prev_cursor_on: false,
      error: None,
    };

    ko.redraw_screen();
    ko
  }

  pub fn cls(&mut self, color: impl Into<u8> + Copy) {
    if color.into() as usize >= self.palette.len() {
      return;
    }

    for i in 0..WIDTH {
      for j in 0..HEIGHT {
        self.draw_dot(i, j, color);
      }
    }
  }

  #[inline]
  pub fn draw_dot(&mut self, x: u32, y: u32, color: impl Into<u8> + Copy) {
    if x >= WIDTH || y >= HEIGHT {
      return;
    }

    self.video[(x + y * WIDTH) as usize] = self.palette[color.into() as usize];
  }

  #[inline]
  pub fn draw_dot_cond(&mut self, x: u32, y: u32, color: impl Into<u8> + Copy, cond: bool) {
    if x >= WIDTH || y >= HEIGHT {
      return;
    }

    self.video[(x + y * WIDTH) as usize] = self.palette[color.into() as usize] * cond as u32 + self.video[(x + y * WIDTH) as usize] * (!cond) as u32;
  }

  pub fn draw_rect(&mut self, x: u32, y: u32, width: u32, height: u32, color: impl Into<u8> + Copy) {
    for i in x..x + width {
      for j in y..y + height {
        self.draw_dot(i, j, color);
      }
    }
  }

  fn draw_line_impl((x1, y1): (i32, i32), (x2, y2): (i32, i32), color: impl Into<u8> + Copy, mut draw_dot: impl FnMut(u32, u32, u8)) {
    let dx = (x2 - x1).abs();
    let dy = (y2 - y1).abs();
    let sx = if x1 < x2 { 1 } else { -1 };
    let sy = if y1 < y2 { 1 } else { -1 };
    let mut err = dx - dy;
    let mut x = x1;
    let mut y = y1;
    loop {
      draw_dot(x as u32, y as u32, color.into());
      if x == x2 && y == y2 {
        break;
      }
      let e2 = 2 * err;
      if e2 > -dy {
        err -= dy;
        x += sx;
      }
      if e2 < dx {
        err += dx;
        y += sy;
      }
    }
  }

  pub fn draw_poly(&mut self, vertices: Vec<(i32, i32)>, color: impl Into<u8> + Copy) -> Result<(), String> {
    if vertices.len() < 3 {
      return Err(format!("Polygon must have at least 3 vertices, got {}", vertices.len()));
    }

    let mut min_x = i32::MAX;
    let mut min_y = i32::MAX;
    let mut max_x = i32::MIN;
    let mut max_y = i32::MIN;

    for i in 0..vertices.len() {
      let (x, y) = vertices[i];
      if x < min_x {
        min_x = x;
      }
      if y < min_y {
        min_y = y;
      }
      if x > max_x {
        max_x = x;
      }
      if y > max_y {
        max_y = y;
      }
    }

    let mut polybuf = vec![false; (max_x - min_x + 1) as usize * (max_y - min_y + 1) as usize];
    let mut scanline_info = vec![0; (max_y - min_y) as usize + 1];

    for i in 0..vertices.len() {
      let (x1, y1) = vertices[i];
      let (x2, y2) = vertices[(i + 1) % vertices.len()];
      if y1 == y2 {
        continue;
      }

      // one pixel per line
      for y in min(y1, y2)..=max(y1, y2) {
        let x = x1 + (x2 - x1) * (y - y1) / (y2 - y1);
        polybuf[((x - min_x) + (y - min_y) * (max_x - min_x + 1)) as usize] = true;
        scanline_info[(y - min_y) as usize] = max(x - min_x, scanline_info[(y - min_y) as usize]);
      }
    }

    for y in 0..max_y - min_y + 1 {
      let mut on = false;
      for x in 0..max_x - min_x + 1 {
        if polybuf[(x + y * (max_x - min_x + 1)) as usize] {
          on = !on;
        }

        if on {
          self.draw_dot((x + min_x) as u32, (y + min_y) as u32, color);
        }

        if x == scanline_info[y as usize] {
          break;
        }
      }
    }

    Ok(())
  }

  pub fn draw_line(&mut self, (x1, y1): (i32, i32), (x2, y2): (i32, i32), color: impl Into<u8> + Copy) {
    Self::draw_line_impl((x1, y1), (x2, y2), color, |x, y, color| self.draw_dot(x, y, color));
  }

  pub fn draw_text(&mut self, text: &str, mut x: u32, y: u32, color: impl Into<u8> + Copy, clear_background: Option<impl Into<u8> + Copy>) {
    for char in text.chars() {
      let char = char as usize;
      if char >= 256 {
        continue;
      }

      let char_info = self.char_info[char];
      if char_info == Character::invalid() {
        x += 5;
        continue;
      }

      let top_left_x = char_info.top_left_x;
      let top_left_y = char_info.top_left_y;
      let bottom_right_x = char_info.bottom_right_x;
      let bottom_right_y = char_info.bottom_right_y;

      if let Some(clear_background) = clear_background {
        for j in -1..(bottom_right_y - top_left_y + 1) as i32 {
          self.draw_dot((0 + x as i32) as u32, (j + y as i32) as u32, clear_background);
          self.draw_dot((1 + x as i32) as u32, (j + y as i32) as u32, clear_background);
          self.draw_dot((2 + x as i32) as u32, (j + y as i32) as u32, clear_background);
          self.draw_dot((3 + x as i32) as u32, (j + y as i32) as u32, clear_background);
        }
      }

      for j in 0..bottom_right_y - top_left_y {
        self.draw_dot_cond(0 + x, j + y, color, self.font_pixels[(0 + top_left_x) as usize][(j + top_left_y) as usize]);
        self.draw_dot_cond(1 + x, j + y, color, self.font_pixels[(1 + top_left_x) as usize][(j + top_left_y) as usize]);
        self.draw_dot_cond(2 + x, j + y, color, self.font_pixels[(2 + top_left_x) as usize][(j + top_left_y) as usize]);
        self.draw_dot_cond(3 + x, j + y, color, self.font_pixels[(3 + top_left_x) as usize][(j + top_left_y) as usize]);
      }

      x += 5;
    }
  }

  pub fn draw_text_with_shadow(
    &mut self,
    text: &str,
    x: u32,
    y: u32,
    color: impl Into<u8> + Copy,
    shadow_color: impl Into<u8> + Copy,
    clear_background: Option<impl Into<u8> + Copy>,
  ) {
    self.draw_text(text, x + 1, y + 1, shadow_color, clear_background);
    self.draw_text(text, x, y, color, None::<u8>);
  }

  pub fn on_key(&mut self, event: Event) {
    match event {
      Event::KeyDown { keycode, keymod, .. } => {
        match keycode {
          Some(Keycode::Tab) => {
            if keymod.contains(sdl2::keyboard::Mod::LCTRLMOD) {
              self.screen = (self.screen + 1) % 2;
              self.redraw_screen();
            } else if self.current_line.is_empty() && self.line_cursor < self.basic.program.len() as u32 {
              self.current_line = self.basic.program[self.line_cursor as usize].contents.clone();
              self.cursor = self.current_line.len() as u32;
            }
          }
          Some(Keycode::Backspace) => {
            match self.screen {
              BASIC_SCREEN => {
                if self.cursor > 0 {
                  self.current_line.remove(self.cursor as usize - 1);
                  self.cursor -= 1;
                }
              }
              _ => {}
            }
          }
          Some(Keycode::Delete) => {
            match self.screen {
              BASIC_SCREEN => {
                if self.cursor < self.current_line.len() as u32 {
                  self.current_line.remove(self.cursor as usize);
                }
              }
              _ => {}
            }
          }
          Some(Keycode::Home) => {
            match self.screen {
              BASIC_SCREEN => {
                self.cursor = 0;
              }
              _ => {}
            }
          }
          Some(Keycode::End) => {
            match self.screen {
              BASIC_SCREEN => {
                self.cursor = self.current_line.len() as u32;
              }
              _ => {}
            }
          }
          Some(Keycode::Left) => {
            match self.screen {
              BASIC_SCREEN => {
                if self.cursor > 0 {
                  self.cursor -= 1;
                }
              }
              _ => {}
            }
          }
          Some(Keycode::Right) => {
            match self.screen {
              BASIC_SCREEN => {
                if self.cursor < self.current_line.len() as u32 {
                  self.cursor += 1;
                }
              }
              _ => {}
            }
          }
          Some(Keycode::Up) => {
            match self.screen {
              BASIC_SCREEN => {
                if self.line_cursor > 0 {
                  self.line_cursor -= 1;
                  self.redraw_screen();
                }
              }
              _ => {}
            }
          }
          Some(Keycode::Down) => {
            match self.screen {
              BASIC_SCREEN => {
                if self.line_cursor < self.basic.program.len() as u32 - 1 {
                  self.line_cursor += 1;
                  self.redraw_screen();
                }
              }
              _ => {}
            }
          }
          Some(Keycode::Return) => {
            match self.screen {
              BASIC_SCREEN => {
                let res = self.basic.add_line(self.current_line.clone());
                if let Err(error) = res {
                  self.error = Some(error);
                } else if let Ok(Some(node)) = res {
                  self.interpret(node).unwrap();
                } else {
                  self.error = None;
                  self.current_line.clear();
                }
                self.cursor = 0;
                self.redraw_screen();
              }
              _ => {}
            }
          }
          _ => {}
        }
      }
      _ => {}
    }
  }

  pub fn on_text_input(&mut self, event: Event) {
    if let Event::TextInput { text, .. } = event {
      match self.screen {
        BASIC_SCREEN => {
          self.current_line.insert_str(self.cursor as usize, text.as_str());
          self.cursor += text.len() as u32;
        }
        _ => {}
      }
    }
  }

  fn redraw_screen(&mut self) {
    self.printed_text.clear();
    match self.screen {
      BASIC_SCREEN => {
        self.cls(Sweetie16::DarkGray);
        self.cursor = 0;
        self.scroll = 0;
        for i in self.basic.program.len() - min(TEXT_HEIGHT as usize - 1, self.basic.program.len())..self.basic.program.len() {
          let mut display = self.basic.program[i].contents.clone();
          if i == self.line_cursor as usize {
            display = String::from("> ") + display.as_str();
          }
          self.draw_text_with_shadow(display.as_str(), 3, 3 + i as u32 * 12, Sweetie16::White, Sweetie16::Black, None::<u8>)
        }

        if let Some(error) = &self.error {
          self.draw_text_with_shadow(error.clone().as_str(), 3, HEIGHT - 27, Sweetie16::Red, Sweetie16::Black, None::<u8>)
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
        self.draw_rect(0, HEIGHT - 15, WIDTH, 15, Sweetie16::DarkGray);
        self.draw_text_with_shadow(("basic: ".to_string() + self.current_line.clone().as_str()).as_str(), 3, HEIGHT - 13, Sweetie16::White, Sweetie16::Black, None::<u8>);
        let cursor_on = SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_millis() % 1000 < 500;
        if cursor_on {
          self.draw_text_with_shadow("       _", 3 + self.cursor * 5, HEIGHT - 12, Sweetie16::White, Sweetie16::Black, None::<u8>);
        }
      }
      EXEC_SCREEN => {}
      _ => panic!("Unknown screen {}", self.screen)
    }
  }

  pub fn execute_code(&mut self) -> Result<(), String> {
    if self.screen == EXEC_SCREEN {
      while !self.basic.refresh {
        if self.basic.line_no >= self.basic.program.len() {
          break;
        }
        self.exec_current_line()?;
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
        self.draw_text_with_shadow(
          self.printed_text[i as usize].clone().as_str(),
          2,
          i * 12 + 2,
          Sweetie16::White,
          Sweetie16::DarkGray,
          Some(Sweetie16::Black),
        );
      }
    } else {
      self.printed_text.push(String::from(text));
      self.draw_text_with_shadow(
        self.printed_text[self.printed_text.len() - 1].clone().as_str(),
        2,
        (self.printed_text.len() - 1) as u32 * 12 + 2,
        Sweetie16::White,
        Sweetie16::DarkGray,
        Some(Sweetie16::Black),
      );
    }
  }

  pub fn palette_idx_from_value(value: &Value) -> Result<u8, String> {
    Ok(match value {
      Value::Integer(num) => *num as u8,
      Value::String(str) => {
        match str.as_str() {
          "orange" | "org" => {
            Sweetie16::Orange.into()
          }
          "red" => {
            Sweetie16::Red.into()
          }
          "yellow" | "yel" => {
            Sweetie16::Yellow.into()
          }
          "green" | "grn" => {
            Sweetie16::DarkGreen.into()
          }
          "blue" | "blu" => {
            Sweetie16::DarkBlue.into()
          }
          "light_blue" => {
            Sweetie16::LightBlue.into()
          }
          "deep_blue" => {
            Sweetie16::DeepBlue.into()
          }
          "light_green" => {
            Sweetie16::LightGreen.into()
          }
          "teal" => {
            Sweetie16::Teal.into()
          }
          "aqua" => {
            Sweetie16::Aqua.into()
          }
          "dark_gray" => {
            Sweetie16::DarkGray.into()
          }
          "medium_gray" => {
            Sweetie16::MediumGray.into()
          }
          "light_gray" => {
            Sweetie16::LightGray.into()
          }
          "purple" | "pur" => {
            Sweetie16::Purple.into()
          }
          "black" | "blk" => {
            Sweetie16::Black.into()
          }
          "white" | "wht" => {
            Sweetie16::White.into()
          }
          _ => return Err(format!("Unknown color {}", str))
        }
      }
      _ => return Err(format!("Expected integer, got {:?}", value))
    })
  }

  pub fn interpret(&mut self, node: Node) -> Result<Value, String> {
    match node {
      Node::Integer(num) => Ok(Value::Integer(num)),
      Node::Float(num) => Ok(Value::Float(num)),
      Node::String(string) => Ok(Value::String(string.clone())),
      Node::VarGet(name) => {
        if let Some(value) = self.basic.vars.get(name.as_str()) {
          Ok(value.clone())
        } else {
          Err(format!("Variable {} not found!", name))
        }
      }
      Node::Assign { name, value } => {
        let value = self.interpret(*value)?;
        self.basic.vars.insert(name.clone(), value.clone());
        Ok(value)
      }
      Node::For { name, start, end, step } => {
        if let Some(_value) = self.basic.vars.get(name.as_str()) {
          return Err(format!("Variable {} already exists!", name));
        }

        let start = self.interpret(*start)?;
        let end = self.interpret(*end)?;
        let step = self.interpret(*step)?;

        self.basic.vars.insert(name.clone(), start.clone());
        self.basic.for_stack.push((self.basic.line_no, end, step));

        Ok(Value::Nil)
      }
      Node::If { cond, then, else_ } => {
        let cond = self.interpret(*cond)?;
        if cond.is_truthy() {
          self.interpret(*then)
        } else {
          self.interpret(*else_)
        }
      }
      Node::BinOp { op, left, right } => {
        let left = self.interpret(*left)?;
        let right = self.interpret(*right)?;

        match op {
          Token::Add => {
            match (&left, &right) {
              (Value::Integer(left), Value::Integer(right)) => Ok(Value::Integer(left + right)),
              (Value::Float(left), Value::Float(right)) => Ok(Value::Float(left + right)),
              (Value::Integer(left), Value::Float(right)) => Ok(Value::Float(*left as f64 + right)),
              (Value::Float(left), Value::Integer(right)) => Ok(Value::Float(left + *right as f64)),
              (Value::String(left), Value::String(right)) => Ok(Value::String(left.clone() + right.clone().as_str())),
              _ => Err(format!("Cannot compare {:?} and {:?} with op {:?}", left, right, op))
            }
          }
          Token::Sub => {
            match (&left, &right) {
              (Value::Integer(left), Value::Integer(right)) => Ok(Value::Integer(left - right)),
              (Value::Float(left), Value::Float(right)) => Ok(Value::Float(left - right)),
              (Value::Integer(left), Value::Float(right)) => Ok(Value::Float(*left as f64 - right)),
              (Value::Float(left), Value::Integer(right)) => Ok(Value::Float(left - *right as f64)),
              _ => Err(format!("Cannot compare {:?} and {:?} with op {:?}", left, right, op))
            }
          }
          Token::Percent => {
            match (&left, &right) {
              (Value::Integer(left), Value::Integer(right)) => Ok(Value::Integer(left % right)),
              (Value::Float(left), Value::Float(right)) => Ok(Value::Float(left % right)),
              (Value::Integer(left), Value::Float(right)) => Ok(Value::Float(*left as f64 % right)),
              (Value::Float(left), Value::Integer(right)) => Ok(Value::Float(left % *right as f64)),
              _ => Err(format!("Cannot compare {:?} and {:?} with op {:?}", left, right, op))
            }
          }
          Token::Mul => {
            match (&left, &right) {
              (Value::Integer(left), Value::Integer(right)) => Ok(Value::Integer(left * right)),
              (Value::Float(left), Value::Float(right)) => Ok(Value::Float(left * right)),
              (Value::Integer(left), Value::Float(right)) => Ok(Value::Float(*left as f64 * right)),
              (Value::Float(left), Value::Integer(right)) => Ok(Value::Float(left * *right as f64)),
              _ => Err(format!("Cannot compare {:?} and {:?} with op {:?}", left, right, op))
            }
          }
          Token::Div => {
            match (&left, &right) {
              (Value::Integer(left), Value::Integer(right)) => Ok(Value::Integer(left / right)),
              (Value::Float(left), Value::Float(right)) => Ok(Value::Float(left / right)),
              (Value::Integer(left), Value::Float(right)) => Ok(Value::Float(*left as f64 / right)),
              (Value::Float(left), Value::Integer(right)) => Ok(Value::Float(left / *right as f64)),
              _ => Err(format!("Cannot compare {:?} and {:?} with op {:?}", left, right, op))
            }
          }
          Token::Lt => {
            Ok(Value::Integer((left.comparison_value()? < right.comparison_value()?) as i64))
          }
          Token::Gt => {
            Ok(Value::Integer((left.comparison_value()? > right.comparison_value()?) as i64))
          }
          Token::Gte => {
            Ok(Value::Integer((left.comparison_value()? > right.comparison_value()? || (left.comparison_value()? - right.comparison_value()?).abs() < 0.0000001) as i64))
          }
          Token::Lte => {
            Ok(Value::Integer((left.comparison_value()? < right.comparison_value()? || (left.comparison_value()? - right.comparison_value()?).abs() < 0.0000001) as i64))
          }
          _ => Err(format!("Cannot compare {:?} and {:?} with op {:?}", left, right, op))
        }
      }
      Node::UnOp { op, right } => {
        match op {
          Token::Exclamation => {
            let right = self.interpret(*right)?;
            Ok(Value::Integer(!right.is_truthy() as i64))
          }
          Token::Sub => {
            let right = self.interpret(*right)?;
            match right {
              Value::Integer(num) => Ok(Value::Integer(-num)),
              Value::Float(num) => Ok(Value::Float(-num)),
              _ => Err(format!("Cannot negate {:?}", right))
            }
          }
          Token::Add => {
            let right = self.interpret(*right)?;
            match right {
              Value::Integer(num) => Ok(Value::Integer(num)),
              Value::Float(num) => Ok(Value::Float(num)),
              _ => Err(format!("Cannot negate {:?}", right))
            }
          }
          _ => {
            return Err(format!("Unknown unary operator {:?}", op));
          }
        }
      }
      Node::BuiltinCommand { name, args } => {
        match name.as_str() {
          "refresh" => {
            if args.len() != 0 {
              return Err(format!("Expected 0 arguments, got {}", args.len()));
            }

            self.basic.refresh = true;
            Ok(Value::Nil)
          }
          "rnd" => {
            if args.len() != 2 {
              return Err(format!("Expected 2 arguments, got {}", args.len()));
            }

            let mut rng = rand::thread_rng();

            let min = self.interpret(args[0].clone())?.to_float()?;
            let max = self.interpret(args[1].clone())?.to_float()?;

            Ok(Value::Float(rng.gen_range(min..max)))
          }
          "gosub" => {
            if args.len() != 1 {
              return Err(format!("Expected 1 argument, got {}", args.len()));
            }

            let line_no = match self.interpret(args[0].clone())? {
              Value::Integer(num) => {
                self.basic.program.iter().position(|x| x.line_no == num as usize).ok_or(format!("Gosub: Could not find line {}", num))?
              }
              _ => return Err(format!("Expected integer, got {:?}", self.interpret(args[0].clone())?))
            };

            self.basic.call_stack.push(self.basic.line_no);
            self.basic.line_no = line_no;
            self.basic.no_increment_instr_counter = true;
            Ok(Value::Nil)
          }
          "delay" => {
            if args.len() != 1 {
              return Err(format!("Expected 1 argument, got {}", args.len()));
            }

            let value = self.interpret(args[0].clone())?;
            match value {
              Value::Integer(num) => {
                std::thread::sleep(std::time::Duration::from_millis(num as u64));
                Ok(Value::Nil)
              }
              Value::Float(num) => {
                std::thread::sleep(std::time::Duration::from_millis(num as u64));
                Ok(Value::Nil)
              }
              _ => Err(format!("Expected integer or float, got {:?}", value))
            }
          }
          "sin" => {
            if args.len() != 1 {
              return Err(format!("Expected 1 argument, got {}", args.len()));
            }

            let value = self.interpret(args[0].clone())?;
            match value {
              Value::Integer(num) => Ok(Value::Float((num as f64).sin())),
              Value::Float(num) => Ok(Value::Float(num.sin())),
              _ => Err(format!("Expected integer or float, got {:?}", value))
            }
          }
          "cos" => {
            if args.len() != 1 {
              return Err(format!("Expected 1 argument, got {}", args.len()));
            }

            let value = self.interpret(args[0].clone())?;
            match value {
              Value::Integer(num) => Ok(Value::Float((num as f64).cos())),
              Value::Float(num) => Ok(Value::Float(num.cos())),
              _ => Err(format!("Expected integer or float, got {:?}", value))
            }
          }
          "time" => {
            if args.len() != 0 {
              return Err(format!("Expected 0 arguments, got {}", args.len()));
            }

            let a = secs_since_start();
            Ok(Value::Float(a))
          }
          "end" => {
            if args.len() != 0 {
              return Err(format!("Expected 0 arguments, got {}", args.len()));
            }

            self.basic.line_no = self.basic.program.len();
            Ok(Value::Nil)
          }
          "print" => {
            if args.len() != 1 {
              return Err(format!("Expected 1 argument, got {}", args.len()));
            }

            let value = self.interpret(args[0].clone())?;
            self.print(value.to_string(true));
            Ok(Value::Nil)
          }
          "str" => {
            if args.len() != 1 {
              return Err(format!("Expected 1 argument, got {}", args.len()));
            }

            let value = self.interpret(args[0].clone())?;
            Ok(Value::String(value.to_string(false)))
          }
          "chr" => {
            if args.len() != 1 {
              return Err(format!("Expected 1 argument, got {}", args.len()));
            }

            let value = self.interpret(args[0].clone())?;
            match value {
              Value::Integer(num) => {
                if num < 0 || num > 255 {
                  return Err(format!("Expected integer between 0 and 255, got {}", num));
                }
                Ok(Value::String((num as u8 as char).to_string()))
              }
              _ => Err(format!("Expected integer, got {:?}", value))
            }
          }
          "int" => {
            if args.len() != 1 {
              return Err(format!("Expected 1 argument, got {}", args.len()));
            }

            let value = self.interpret(args[0].clone())?;
            Ok(Value::Integer(value.to_integer()?))
          }
          "poly" => {
            if args.len() < 2 {
              return Err(format!("Expected at least 2 arguments, got {}", args.len()));
            }

            let array_to_vec2i = |array| {
              match array {
                Value::Array(elements) => {
                  if elements.len() != 2 {
                    return Err(format!("Expected array of length 2, got {}", elements.len()));
                  }

                  let x = match elements[0] {
                    Value::Integer(num) => num as i32,
                    Value::Float(num) => num.round() as i32,
                    _ => return Err(format!("Expected integer, got {:?}", elements[0]))
                  };

                  let y = match elements[1] {
                    Value::Integer(num) => num as i32,
                    Value::Float(num) => num.round() as i32,
                    _ => return Err(format!("Expected integer, got {:?}", elements[1]))
                  };

                  Ok((x, y))
                }
                _ => return Err(format!("Expected integer, got {:?}", array))
              }
            };

            if let Value::Array(elements) = self.interpret(args[0].clone())? {
              if elements.len() > 2 && args.len() == 2 {
                let mut points = Vec::<(i32, i32)>::new();
                for i in 0..elements.len() {
                  let point = array_to_vec2i(elements[i].clone())?;
                  points.push(point);
                }
                let color = Self::palette_idx_from_value(&self.interpret(args[1].clone())?)?;

                self.draw_poly(points, color)?;
                return Ok(Value::Nil);
              }
            }

            if args.len() < 4 {
              return Err(format!("Expected at least 4 arguments, got {}", args.len()));
            }

            let mut points = Vec::<(i32, i32)>::new();
            for i in 0..args.len() - 1 {
              let point = array_to_vec2i(self.interpret(args[i].clone())?)?;
              points.push(point);
            }

            let color = Self::palette_idx_from_value(&self.interpret(args[args.len() - 1].clone())?)?;

            self.draw_poly(points, color)?;
            Ok(Value::Nil)
          }
          "line" => {
            if args.len() != 3 {
              return Err(format!("Expected 3 arguments, got {}", args.len()));
            }

            let (x1, y1) = match self.interpret(args[0].clone())? {
              Value::Array(elements) => {
                if elements.len() != 2 {
                  return Err(format!("Expected array of length 2, got {}", elements.len()));
                }

                let x = match elements[0] {
                  Value::Integer(num) => num as i32,
                  Value::Float(num) => num.round() as i32,
                  _ => return Err(format!("Expected integer, got {:?}", elements[0]))
                };

                let y = match elements[1] {
                  Value::Integer(num) => num as i32,
                  Value::Float(num) => num.round() as i32,
                  _ => return Err(format!("Expected integer, got {:?}", elements[1]))
                };

                (x, y)
              }
              _ => return Err(format!("Expected integer, got {:?}", self.interpret(args[0].clone())?))
            };

            let (x2, y2) = match self.interpret(args[1].clone())? {
              Value::Array(elements) => {
                if elements.len() != 2 {
                  return Err(format!("Expected array of length 2, got {}", elements.len()));
                }

                let x = match elements[0] {
                  Value::Integer(num) => num as i32,
                  Value::Float(num) => num.round() as i32,
                  _ => return Err(format!("Expected integer, got {:?}", elements[0]))
                };

                let y = match elements[1] {
                  Value::Integer(num) => num as i32,
                  Value::Float(num) => num.round() as i32,
                  _ => return Err(format!("Expected integer, got {:?}", elements[1]))
                };

                (x, y)
              }
              _ => return Err(format!("Expected integer, got {:?}", self.interpret(args[1].clone())?))
            };

            let color = Self::palette_idx_from_value(&self.interpret(args[2].clone())?)?;

            self.draw_line((x1, y1), (x2, y2), color);

            Ok(Value::Nil)
          }
          "next" => {
            if args.len() != 1 {
              return Err(format!("Expected 1 argument, got {}", args.len()));
            }

            let name = match &args[0] {
              Node::VarGet(name) => name,
              _ => return Err(format!("Expected variable name, got {:?}", args[0]))
            };

            if let Some((line_no, end, step)) = self.basic.for_stack.pop() {
              let mut value = self.basic.vars.get(name).unwrap().clone();
              match value {
                Value::Integer(ref mut num) => {
                  *num += step.to_integer()?;
                }
                Value::Float(ref mut num) => {
                  *num += step.to_float()?;
                }
                _ => return Err(format!("Expected integer or float, got {:?}", value))
              }

              let step_sign = match step {
                Value::Integer(num) => num.signum() as f64,
                Value::Float(num) => num.signum(),
                _ => return Err(format!("Expected integer or float, got {:?}", step))
              };

              if value.comparison_value()? * step_sign < end.comparison_value()? {
                self.basic.vars.insert(name.clone(), value);
                self.basic.line_no = line_no;
                self.basic.for_stack.push((line_no, end, step));
                Ok(Value::Nil)
              } else {
                self.basic.vars.remove(name);
                Ok(Value::Nil)
              }
            } else {
              Err("Cannot next; for stack is empty!".to_string())
            }
          }
          "cls" => {
            if args.len() > 1 {
              return Err(format!("Expected 0 or 1 arguments, got {}", args.len()));
            }

            let color = if let Some(arg) = args.get(0) {
              match self.interpret(arg.clone())? {
                Value::Integer(num) => num as u8,
                _ => return Err(format!("Expected integer, got {:?}", self.interpret(arg.clone())?))
              }
            } else {
              0u8
            };

            self.cls(color);
            Ok(Value::Nil)
          }
          "loop" => {
            if args.len() != 0 {
              return Err(format!("Expected 0 arguments, got {}", args.len()));
            }

            if self.basic.while_stack.len() == 0 {
              return Err("Cannot loop; while stack is empty!".to_string());
            }

            let line_no = self.basic.while_stack.last().unwrap().clone();
            let cond = match self.basic.program[line_no].node.clone() {
              Node::BuiltinCommand { name, args, } => {
                assert_eq!(name, "while");
                assert_eq!(args.len(), 1);
                self.interpret(args[0].clone()).unwrap()
              }
              _ => return Err(format!("Expected if statement, got {:?}", self.basic.program[line_no].node.clone()))
            };

            if cond.is_truthy() {
              self.basic.line_no = line_no;
              Ok(Value::Nil)
            } else {
              Ok(Value::Nil)
            }
          }
          "while" => {
            if args.len() != 1 {
              return Err(format!("Expected 1 argument, got {}", args.len()));
            }

            let cond = self.interpret(args[0].clone())?;
            if cond.is_truthy() {
              self.basic.while_stack.push(self.basic.line_no);
              Ok(Value::Nil)
            } else {
              Ok(Value::Nil)
            }
          }
          "goto" => {
            if args.len() != 1 {
              return Err(format!("Expected 1 argument, got {}", args.len()));
            }

            let line_no = match self.interpret(args[0].clone())? {
              Value::Integer(num) => {
                self.basic.program.iter().position(|x| x.line_no == num as usize).ok_or(format!("Goto: Could not find line {}", num))?
              }
              _ => return Err(format!("Expected integer, got {:?}", self.interpret(args[0].clone())?))
            };

            self.basic.line_no = line_no;
            self.basic.no_increment_instr_counter = true;
            Ok(Value::Nil)
          }
          "ret" => {
            if args.len() != 0 {
              return Err(format!("Expected 0 arguments, got {}", args.len()));
            }

            if let Some(line_no) = self.basic.call_stack.pop() {
              self.basic.line_no = line_no;
              Ok(Value::Nil)
            } else {
              Err("Cannot return; callstack is empty!".to_string())
            }
          }
          "dot" => {
            if args.len() != 3 {
              return Err(format!("Expected 3 arguments, got {}", args.len()));
            }

            let x = match self.interpret(args[0].clone())? {
              Value::Integer(num) => num as u32,
              Value::Float(num) => num.round() as u32,
              _ => return Err(format!("Expected integer, got {:?}", self.interpret(args[0].clone())?))
            };

            let y = match self.interpret(args[1].clone())? {
              Value::Integer(num) => num as u32,
              Value::Float(num) => num.round() as u32,
              _ => return Err(format!("Expected integer, got {:?}", self.interpret(args[1].clone())?))
            };

            let color = Self::palette_idx_from_value(&self.interpret(args[2].clone())?)?;

            self.draw_dot(x, y, color);
            Ok(Value::Nil)
          }
          "rad" => {
            if args.len() != 1 {
              return Err(format!("Expected 1 argument, got {}", args.len()));
            }

            let value = self.interpret(args[0].clone())?.to_float()?;
            Ok(Value::Float(value.to_radians()))
          }
          "deg" => {
            if args.len() != 1 {
              return Err(format!("Expected 1 argument, got {}", args.len()));
            }

            let value = self.interpret(args[0].clone())?.to_float()?;
            Ok(Value::Float(value.to_degrees()))
          }
          "save" => {
            if args.len() != 1 {
              return Err(format!("Expected 1 argument, got {}", args.len()));
            }

            let filename = match self.interpret(args[0].clone())? {
              Value::String(str) => str,
              _ => return Err(format!("Expected string, got {:?}", self.interpret(args[0].clone())?))
            };

            let mut file = File::create(&filename);
            if let Err(err) = file {
              return Err(format!("Could not create file {}: {}", filename, err));
            }

            let mut file = file.unwrap();
            for line in &self.basic.program {
              if let Err(err) = writeln!(file, "{}", line.contents) {
                return Err(format!("Could not write to file {}: {}", &filename, &err));
              }
            }

            Ok(Value::Nil)
          }
          "load" => {
            if args.len() != 1 {
              return Err(format!("Expected 1 argument, got {}", args.len()));
            }

            let filename = match self.interpret(args[0].clone())? {
              Value::String(str) => str,
              _ => return Err(format!("Expected string, got {:?}", self.interpret(args[0].clone())?))
            };

            let mut file = File::open(&filename);

            if let Err(err) = file {
              return Err(format!("Could not open file {}: {}", filename, err));
            }

            let mut file = file.unwrap();
            let mut buffer = String::new();

            if let Err(err) = file.read_to_string(&mut buffer) {
              return Err(format!("Could not read from file {}: {}", filename, err));
            }

            let program_vec = buffer.split("\n").map(|x| x.to_string()).collect::<Vec<String>>();
            self.basic.program.clear();

            for line in program_vec {
              if line.len() == 0 {
                continue;
              }

              self.basic.add_line(line)?;
            }

            Ok(Value::Nil)
          }
          _ => {
            return Err(format!("Unknown builtin command {}", name));
          }
        }
      }
      Node::End => {
        self.basic.line_no = self.basic.program.len();
        Ok(Value::Nil)
      }
      Node::Nil => {
        Ok(Value::Nil)
      }
      Node::Array(elements) => {
        let mut array = Vec::new();
        for element in elements {
          array.push(self.interpret(element)?);
        }
        Ok(Value::Array(array))
      }
      Node::IndexGet { name, index } => {
        let index = self.interpret(*index)?;
        let index = match index {
          Value::Integer(num) => num as usize,
          _ => return Err(format!("Expected integer, got {:?}", index))
        };

        let array = match self.basic.vars.get(name.as_str()) {
          Some(Value::Array(array)) => array,
          _ => return Err(format!("Expected array, got {:?}", self.basic.vars.get(name.as_str())))
        };

        if index >= array.len() {
          return Err(format!("Index {} out of bounds for array of length {}", index, array.len()));
        }

        Ok(array[index].clone())
      }
      Node::IndexSet { name, index, value } => {
        let index = self.interpret(*index)?;
        let index = match index {
          Value::Integer(num) => num as usize,
          _ => return Err(format!("Expected integer, got {:?}", index))
        };

        let value = self.interpret(*value)?;

        let array = match self.basic.vars.get_mut(name.as_str()) {
          Some(Value::Array(array)) => array,
          _ => return Err(format!("Expected array, got {:?}", self.basic.vars.get(name.as_str())))
        };

        if index >= array.len() {
          return Err(format!("Index {} out of bounds for array of length {}", index, array.len()));
        }

        array[index] = value;
        Ok(Value::Nil)
      }
      Node::EmptyArray(size) => {
        let size = self.interpret(*size)?;
        let size = match size {
          Value::Integer(num) => usize::try_from(num).unwrap(),
          _ => return Err(format!("Expected integer, got {:?}", size))
        };

        let mut array = Vec::new();
        for _ in 0..size {
          array.push(Value::Nil);
        }

        Ok(Value::Array(array))
      }
    }
  }

  pub fn exec_current_line(&mut self) -> Result<Value, String> {
    if self.basic.line_no >= self.basic.program.len() {
      return Err("Program buffer empty!".to_string());
    }

    let res = self.interpret(self.basic.program[self.basic.line_no].node.clone());
    if !self.basic.no_increment_instr_counter {
      self.basic.line_no += 1;
    }
    self.basic.no_increment_instr_counter = false;
    res
  }
}