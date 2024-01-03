extern crate sdl2;
extern crate image;
extern crate rayon;

use std::time::Duration;
use sdl2::pixels::{Color, PixelFormatEnum};
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::render::TextureCreator;
use sdl2::video::WindowContext;

mod complex;
mod mandelbrot;
use mandelbrot::Mandelbrot;


pub fn main() -> Result<(), String> {
    let mut window_witdh = 800;
    let mut window_height = 600;
    let max_iter = 5000;
    let mut mandelbrot = Mandelbrot::new(window_witdh, window_height, max_iter, -100, 0, 500);
    
    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();

    let window = video_subsystem.window("Mandelbrot", window_witdh, window_height)
    .position_centered()
    .resizable()
    .build()
    .unwrap();

    let mut canvas = window.into_canvas().build().unwrap();
    let texture_creator: TextureCreator<WindowContext> = canvas.texture_creator();
    let mut texture: sdl2::render::Texture<'_> = texture_creator.create_texture_streaming(PixelFormatEnum::RGB24, window_witdh, window_height).unwrap();
    canvas.set_draw_color(Color::RGB(0, 255, 255));
    canvas.clear();
    canvas.present();

    let mut event_pump = sdl_context.event_pump().unwrap();
    
    let mut start: Option<(i32,i32)> = None;
    let mut changed = true;

    let mut fullscreen = false;
    
    'running: loop {
        let mouse_x = event_pump.mouse_state().x();
        let mouse_y = event_pump.mouse_state().y();
        for event in event_pump.poll_iter() {
            match event {
                Event::Window { win_event: sdl2::event::WindowEvent::SizeChanged(w, h), .. } => {
                    window_witdh = w as u32;
                    window_height = h as u32;
                    texture = texture_creator.create_texture_streaming(PixelFormatEnum::RGB24, window_witdh, window_height).unwrap();
                    mandelbrot = Mandelbrot::new(window_witdh, window_height, max_iter, mandelbrot.center_x, mandelbrot.center_y, mandelbrot.zoom);
                    changed = true;
                },
                Event::Quit {..} |
                Event::KeyDown { keycode: Some(Keycode::Escape), .. } => {
                    break 'running
                },
                Event::KeyDown { keycode: Some(Keycode::F11), .. } => {
                    fullscreen = !fullscreen;
                },
                Event::MouseWheel { y, .. } => {
                    mandelbrot.zoom(y as i32, mouse_x, mouse_y);
                    changed = true;
                    start = None;
                },
                Event::MouseButtonDown { x, y , .. } => {
                    start = Some((x, y));
                },
                Event::MouseMotion { x, y, ..} => {
                    if let Some(from) = start {
                        mandelbrot.move_center(from.0 as i64 - x as i64, from.1 as i64 - y as i64);
                        changed = true;
                        start = Some((x, y));
                    }
                }
                Event::MouseButtonUp {..} => {
                    start = None;
                }
                _ => {}
            }
        }

        if fullscreen {
            canvas.window_mut().set_fullscreen(sdl2::video::FullscreenType::Desktop).unwrap();
        } else {
            canvas.window_mut().set_fullscreen(sdl2::video::FullscreenType::Off).unwrap();
        }
        
        canvas.clear();
        if changed {
            let img = mandelbrot.calculate_mandelbrot();
            let img_data = img.into_raw();
            texture.update(None, &img_data, window_witdh as usize * 3).unwrap();
            changed = false;
        }
        canvas.copy(&texture, None, None).unwrap();
        canvas.present();
        ::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 200));
    }
    Ok(())
}