use std::cmp::{max, min};
use crate::koneko::{Character, COLOR_PREFIX, HEIGHT, Koneko, WIDTH};

impl Koneko {
  pub fn cls(&mut self, color: impl Into<u8> + Copy) {
    if color.into() as usize >= self.palette.len() {
      return;
    }

    for i in 0..WIDTH {
      for j in 0..HEIGHT {
        self.pixel(i, j, color);
      }
    }

    self.printed_text.clear();
  }

  #[inline]
  pub fn pixel(&mut self, x: i32, y: i32, color: impl Into<u8> + Copy) {
    if x >= WIDTH || y >= HEIGHT || x < 0 || y < 0 {
      return;
    }

    self.video[(x + y * WIDTH) as usize] = self.palette[color.into() as usize];
  }

  #[inline]
  pub fn pixel_cond(&mut self, x: i32, y: i32, color: impl Into<u8> + Copy, cond: bool) {
    if x >= WIDTH || y >= HEIGHT || x < 0 || y < 0 {
      return;
    }

    self.video[(x + y * WIDTH) as usize] = self.palette[color.into() as usize] * cond as u32 + self.video[(x + y * WIDTH) as usize] * (!cond) as u32;
  }

  pub fn rect(&mut self, x: i32, y: i32, width: i32, height: i32, color: impl Into<u8> + Copy) {
    for i in x..x + width {
      for j in y..y + height {
        self.pixel(i, j, color);
      }
    }
  }

  fn line_impl((x1, y1): (i32, i32), (x2, y2): (i32, i32), color: impl Into<u8> + Copy, mut draw_dot: impl FnMut(i32, i32, u8)) {
    let dx = (x2 - x1).abs();
    let dy = (y2 - y1).abs();
    let sx = if x1 < x2 { 1 } else { -1 };
    let sy = if y1 < y2 { 1 } else { -1 };
    let mut err = dx - dy;
    let mut x = x1;
    let mut y = y1;
    loop {
      draw_dot(x as i32, y as i32, color.into());
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

  pub fn poly(&mut self, vertices: Vec<(i32, i32)>, color: impl Into<u8> + Copy) -> Result<(), String> {
    if vertices.len() < 3 {
      return Err(format!("Polygon must have at least 3 vertices, got {}", vertices.len()));
    }

    let mut min_x = i32::MAX;
    let mut min_y = i32::MAX;
    let mut max_x = i32::MIN;
    let mut max_y = i32::MIN;

    for i in 0..vertices.len() {
      let (x, y) = vertices[i];
      if x - 1 < min_x {
        min_x = x - 1;
      }
      if y - 1 < min_y {
        min_y = y - 1;
      }
      if x + 1 > max_x {
        max_x = x + 1;
      }
      if y + 1 > max_y {
        max_y = y + 1;
      }
    }

    let mut polybuf = vec![0; (max_x - min_x + 1) as usize * (max_y - min_y + 1) as usize];
    let mut scanline_end = vec![-1; (max_y - min_y) as usize + 1];
    let mut scanline_begin = vec![(WIDTH + 1) as i32; (max_y - min_y) as usize + 1];

    for i in 0..vertices.len() {
      let (x1, y1) = vertices[i];
      let (x2, y2) = vertices[(i + 1) % vertices.len()];
      if y1 == y2 {
        continue;
      }

      // one pixel per line
      for y in min(y1, y2)..max(y1, y2) {
        let x = x1 + (x2 - x1) * (y - y1) / (y2 - y1);
        polybuf[((x - min_x) + (y - min_y) * (max_x - min_x + 1)) as usize] += 1;
        scanline_end[(y - min_y) as usize] = max(x - min_x, scanline_end[(y - min_y) as usize]);
        scanline_begin[(y - min_y) as usize] = min(x - min_x, scanline_begin[(y - min_y) as usize]);
      }
    }

    for y in 0..max_y - min_y + 1 {
      let mut on = false;
      for x in scanline_begin[y as usize]..scanline_end[y as usize] {
        if polybuf[(x + y * (max_x - min_x + 1)) as usize] % 2 == 1 {
          on = !on;
        }

        if on {
          self.pixel((x + min_x) as i32, (y + min_y) as i32, color);
        }
      }
    }

    Ok(())
  }

  pub fn outline(&mut self, vertices: Vec<(i32, i32)>, color: impl Into<u8> + Copy) -> Result<(), String> {
    if vertices.len() < 3 {
      return Err(format!("Polygon must have at least 3 vertices, got {}", vertices.len()));
    }

    for i in 0..vertices.len() {
      let (x1, y1) = vertices[i];
      let (x2, y2) = vertices[(i + 1) % vertices.len()];
      self.line((x1, y1), (x2, y2), color);
    }

    Ok(())
  }

  pub fn line(&mut self, (x1, y1): (i32, i32), (x2, y2): (i32, i32), color: impl Into<u8> + Copy) {
    Self::line_impl((x1, y1), (x2, y2), color, |x, y, color| self.pixel(x, y, color));
  }

  fn one_digit_hex(it: u8) -> Option<u8> {
    match it {
      b'0'..=b'9' => Some(it - b'0'),
      b'a'..=b'f' => Some(it - b'a' + 10),
      b'A'..=b'F' => Some(it - b'A' + 10),
      _ => None,
    }
  }

  pub fn text_impl(&mut self, text: &str, mut x: i32, y: i32, shadow: bool, color: impl Into<u8> + Copy, clear_background: Option<impl Into<u8> + Copy>) {
    let mut color = color.into();
    let orig_color = color.into();
    let mut prev_char = b'\0';
    for char in text.bytes() {
      if char == COLOR_PREFIX {
        prev_char = char;
        continue;
      }

      if prev_char == COLOR_PREFIX {
        prev_char = char;
        if let Some(new_color) = Self::one_digit_hex(char) {
          color = new_color;
          continue;
        } else if char == b'r' {
          color = orig_color;
          continue;
        }
      }
      prev_char = char;

      if shadow {
        color = orig_color;
      }

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
          for i in -1..(bottom_right_x - top_left_x + 1) as i32 {
            self.pixel((i + x as i32) as i32, (j + y as i32) as i32, clear_background);
          }
        }
      }

      for j in 0..bottom_right_y - top_left_y {
        for i in 0..bottom_right_x - top_left_x {
          self.pixel_cond(
            i + x,
            j + y,
            color,
            self.font[(i + top_left_x) as usize][(j + top_left_y) as usize]
          );
        }
      }

      x += bottom_right_x - top_left_x + 1;
    }
  }
  
  pub fn width(&self, text: &str) -> i32 {
    let mut width = 0;
    let mut prev_char = b'\0';
    for char in text.bytes() {
      if char == COLOR_PREFIX {
        prev_char = char;
        continue;
      }

      if prev_char == COLOR_PREFIX {
        prev_char = char;
        if let Some(_) = Self::one_digit_hex(char) {
          continue;
        } else if char == b'r' {
          continue;
        }
      }

      prev_char = char;

      let char = char as usize;

      let char_info = self.char_info[char];
      if char_info == Character::invalid() {
        width += 5;
        continue;
      }

      let top_left_x = char_info.top_left_x;
      let bottom_right_x = char_info.bottom_right_x;

      width += bottom_right_x - top_left_x + 1;
    }
    width
  }

  pub fn text(
    &mut self,
    text: &str,
    x: i32,
    y: i32,
    color: impl Into<u8> + Copy,
    shadow_color: Option<impl Into<u8> + Copy>,
    clear_background: Option<impl Into<u8> + Copy>,
  ) {
    if let Some(shadow_color) = shadow_color {
      self.text_impl(text, x + 1, y + 1, true, shadow_color, clear_background);
    }
    self.text_impl(text, x, y, false, color, None::<u8>);
  }
}