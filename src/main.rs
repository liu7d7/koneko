use std::ffi::c_void;

use sdl2::pixels::PixelFormatEnum;
use sdl2::surface::Surface;
use sdl2::sys::{SDL_memcpy, size_t};

use crate::koneko::Koneko;

pub mod basic;
pub mod csv;
pub mod koneko;
pub mod palette;

fn run_koneko(ko: &mut Koneko) {
  extern crate sdl2;

  use sdl2::event::Event;
  use std::time::Duration;

  let sdl_context = sdl2::init().unwrap();
  let video_subsystem = sdl_context.video().unwrap();

  const WINDOW_WIDTH: u32 = koneko::WIDTH * 3;
  const WINDOW_HEIGHT: u32 = koneko::HEIGHT * 3;

  let window = video_subsystem
    .window("koneko", WINDOW_WIDTH, WINDOW_HEIGHT)
    .position_centered()
    .build()
    .unwrap();

  let mut canvas = window.into_canvas().build().unwrap();
  let mut event_pump = sdl_context.event_pump().unwrap();
  let surface = Surface::new(koneko::WIDTH, koneko::HEIGHT, PixelFormatEnum::ABGR32).unwrap();
  'running: loop {
    for event in event_pump.poll_iter() {
      match event {
        Event::Quit { .. } => break 'running,
        Event::KeyDown { .. } => {
          ko.on_key(event);
        }
        Event::TextInput { .. } => {
          ko.on_text_input(event);
        }
        _ => {}
      }
    }

    let begin = std::time::SystemTime::now()
      .duration_since(std::time::UNIX_EPOCH)
      .unwrap()
      .as_millis();
    let a = ko.execute_code();
    if let Ok(_val) = a {
      // println!("val: {}", val);
    } else if let Err(e) = a {
      ko.print(format!("Error: {}", e));
    }
    ko.draw_screen();

    unsafe {
      SDL_memcpy(
        (*surface.raw()).pixels,
        ko.video.as_ptr() as *const c_void,
        koneko::WIDTH as size_t * koneko::HEIGHT as size_t * 4,
      );
    }

    canvas
      .copy(
        &surface.as_texture(&canvas.texture_creator()).unwrap(),
        None,
        None,
      )
      .unwrap();

    canvas.present();
    let end = std::time::SystemTime::now()
      .duration_since(std::time::UNIX_EPOCH)
      .unwrap()
      .as_millis();
    const SIXTY: u32 = 1_000_000_000u32 / 60;
    let nanos = ((end - begin) * 1_000_000) as u32;
    if nanos < SIXTY {
      std::thread::sleep(Duration::new(0, SIXTY - nanos));
    }
  }
}

fn main() {
  run_koneko(&mut Koneko::new(palette::sweetie_16(), "font.png"));
}
