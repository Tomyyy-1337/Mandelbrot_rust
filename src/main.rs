extern crate sdl2;
extern crate image;
extern crate rayon;
use std::time::Duration;

use sdl2::pixels::{Color, PixelFormatEnum};
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::render::TextureCreator;
use sdl2::video::WindowContext;
use sdl2::video::FullscreenType::{Desktop, Off};
use sdl2::ttf::Font;

mod complex;
mod mandelbrot;
use mandelbrot::Mandelbrot;

fn main() -> Result<(), String> {
    let mut window_witdh = 800;
    let mut window_height = 600;
    let max_iter = 5000;
    let mut mandelbrot = Mandelbrot::new(window_witdh, window_height, max_iter, -100, 0, 250);
    
    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();

    let ttf_context = sdl2::ttf::init().map_err(|e| e.to_string()).unwrap();
    let mut font: Font = ttf_context.load_font("font/VCR_OSD_MONO.ttf", 128).unwrap();
    font.set_style(sdl2::ttf::FontStyle::NORMAL);

    let window = video_subsystem.window("Mandelbrot", window_witdh, window_height)
    .position_centered()
    .resizable()
    .build()
    .unwrap();

    let mut canvas = window.into_canvas().build().unwrap();

    let texture_creator = canvas.texture_creator();
    let mut texture: sdl2::render::Texture<'_> = texture_creator.create_texture_streaming(PixelFormatEnum::RGB24, window_witdh, window_height).unwrap();
    canvas.set_draw_color(Color::RGB(0, 255, 255));
    canvas.clear();
    canvas.present();

    let mut event_pump = sdl_context.event_pump().unwrap();
    
    let mut start: Option<(i32,i32)> = None;
    let mut changed = true;

    let mut fullscreen = false;

    let mut real = String::new();
    let mut imag = String::new();
    
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
                Event::KeyDown { keycode: Some(Keycode::R), .. } => {
                    mandelbrot = Mandelbrot::new(window_witdh, window_height, max_iter, -100, 0, 250);
                    changed = true;
                },
                Event::KeyDown { keycode: Some(Keycode::Up), .. } => {
                    mandelbrot.increase_max_iter(mandelbrot.max_iter as i32);
                    changed = true;
                },
                Event::KeyDown { keycode: Some(Keycode::Down), .. } => {
                    mandelbrot.increase_max_iter(mandelbrot.max_iter as i32 / -2);
                    changed = true;
                },
                Event::KeyDown { keycode: Some(Keycode::Space), .. } => {
                    save_image(&mandelbrot, window_witdh, window_height);
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

        canvas.window_mut().set_fullscreen(if fullscreen {Desktop} else {Off}).unwrap();
        
        canvas.clear();
        if changed {
            let img: image::ImageBuffer<image::Rgb<u8>, Vec<u8>> = mandelbrot.calculate_mandelbrot();
            let img_data = img.into_raw();
            texture.update(None, &img_data, window_witdh as usize * 3).unwrap();
            changed = false;
        } else {
            let stepsize = 1.0 / mandelbrot.zoom as f64;
            real = format_float((mandelbrot.center_x + mouse_x as i64 - window_witdh as i64 / 2) as f64 * stepsize);
            imag = format_float(-(mandelbrot.center_y + mouse_y as i64 - window_height as i64 / 2) as f64 * stepsize);

            ::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 120));
        }
        
        canvas.copy(&texture, None, None).unwrap();
        draw_text(5, 28, &format!("Max Iterations: {}", mandelbrot.max_iter), &mut canvas, &texture_creator, &font);
        draw_text(5, 51, &format!("Zoom: {:.2}x", mandelbrot.zoom as f32 / 400.0), &mut canvas, &texture_creator, &font);
        draw_text(5, 5, &format!("C = {} + {}i", real, imag), &mut canvas, &texture_creator, &font); 
        canvas.present();
    }
    Ok(())
}

fn save_image(mandelbrot: &Mandelbrot, window_witdh: u32, window_height: u32) {
    let _ = std::fs::create_dir("img");
    let mut mandelbrot_high_res = mandelbrot.change_size(window_witdh, window_height);
    let img: image::ImageBuffer<image::Rgb<u8>, Vec<u8>> = mandelbrot_high_res.calculate_mandelbrot();
    let real = mandelbrot.center_x as f64 / mandelbrot.zoom as f64;
    let imag = mandelbrot.center_y as f64 / mandelbrot.zoom as f64;
    let c = format!("{}+{}i", real, imag);
    let zoom = format!("{:.1}x", mandelbrot.zoom as f32 / 400.0);
    let img_name = format!("img/{c}_{zoom}.png");
    match img.save(&img_name) {
        Ok(_) => println!("Saved image: {img_name}"),
        Err(e) => println!("Error saving image: {}", e),
    }
}

fn format_float(n: f64) -> String {
    if n < 0.0 {
        format!("{:.16}", n)
    } else {
        format!(" {:.16}", n)
    }
}

fn draw_text(x: u32, y: u32, text: &str, canvas: &mut sdl2::render::Canvas<sdl2::video::Window>, texture_creator: &TextureCreator<WindowContext>, font: &Font) {
    let surface = font
        .render(text)
        .blended(Color::RGBA(0, 0, 0, 255))
        .map_err(|e| e.to_string()).unwrap();
    let texture = texture_creator.create_texture_from_surface(&surface).unwrap();
    let target = sdl2::rect::Rect::new(x as i32 + 1, y as i32 + 1, 16 * text.len() as u32, 18);
    canvas.copy(&texture, None, Some(target)).unwrap();
    let surface = font
        .render(text)
        .blended(Color::RGBA(255, 255, 255, 255))
        .map_err(|e| e.to_string()).unwrap();
    let texture = texture_creator.create_texture_from_surface(&surface).unwrap();
    let target = sdl2::rect::Rect::new(x as i32, y as i32, 16 * text.len() as u32, 18);
    canvas.copy(&texture, None, Some(target)).unwrap();
}