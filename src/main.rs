use std::ffi::c_void;
use sdl2::keyboard::Keycode;

use sdl2::pixels::PixelFormatEnum;
use sdl2::surface::Surface;
use sdl2::sys::{SDL_memcpy, size_t};
use sdl2::video::FullscreenType;

use crate::koneko::Koneko;

pub mod lex_parse_basic;
pub mod csv;
pub mod koneko;
pub mod palette;
pub mod koneko_basic;
pub mod koneko_draw;

fn run_koneko(ko: &mut Koneko) {
  extern crate sdl2;

  use sdl2::event::Event;

  let sdl_context = sdl2::init().unwrap();
  let video_subsystem = sdl_context.video().unwrap();

  let window = video_subsystem
    .window("koneko", koneko::WIDTH as u32 * 3, koneko::HEIGHT as u32 * 3)
    .position_centered()
    .resizable()
    .build()
    .unwrap();

  let mut canvas = window.into_canvas().build().unwrap();
  let mut event_pump = sdl_context.event_pump().unwrap();
  let surface = Surface::new(koneko::WIDTH as u32, koneko::HEIGHT as u32, PixelFormatEnum::ABGR32).unwrap();
  'running: loop {
    for event in event_pump.poll_iter() {
      match event {
        Event::Quit { .. } => break 'running,
        Event::KeyDown { keycode: Some(Keycode::F11), .. } => {
          if let FullscreenType::Desktop = canvas.window().fullscreen_state() {
            canvas.window_mut().set_fullscreen(FullscreenType::Off).unwrap();
          } else {
            canvas.window_mut().set_fullscreen(FullscreenType::Desktop).unwrap();
          }
        }
        Event::KeyDown { .. } => {
          ko.on_key(event);
        }
        Event::KeyUp { .. } => {
          ko.on_key(event);
        }
        Event::TextInput { .. } => {
          ko.on_text_input(event);
        }
        _ => {}
      }
    }
    let a = ko.execute_code();
    if let Ok(_val) = a {

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
  }
}

fn main() {

  run_koneko(&mut Koneko::new(palette::sweetie_16(), "font.png"));
}
