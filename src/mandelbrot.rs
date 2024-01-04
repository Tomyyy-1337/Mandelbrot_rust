use std::collections::HashMap;
use image::{ImageBuffer, Rgb};
use rayon::iter::IntoParallelIterator;
use rayon::iter::ParallelIterator;
use crate::complex::Complex;

pub struct Mandelbrot {
    pub width: u32,
    pub height: u32,
    pub max_iter: u32,
    pub center_x: i64,
    pub center_y: i64,
    pub zoom: u64,
    last_squares: HashMap<Square,SquareResult>,
}

impl Mandelbrot {
    pub fn new(width: u32, height: u32, max_iter: u32, center_x: i64, center_y: i64, zoom: u64) -> Self {
        Mandelbrot {
            width,
            height,
            max_iter,
            center_x,
            center_y,
            zoom,
            last_squares: HashMap::new(),
        }
    }

    pub fn zoom(&mut self, zoom: i32, mouse_x: i32 ,mouse_y: i32) {
        let new_zoom = u64::max((self.zoom as f64 * 1.33f64.powi(zoom)) as u64, 16);
        let x_offset = (mouse_x as i64 - self.width as i64 / 2) * (new_zoom as i64 - self.zoom as i64) / self.zoom as i64;
        let y_offset = (mouse_y as i64 - self.height as i64 / 2) * (new_zoom as i64 - self.zoom as i64) / self.zoom as i64;
        self.center_x = (self.center_x as f64 * new_zoom as f64 / self.zoom as f64) as i64 + x_offset as i64;
        self.center_y = (self.center_y as f64 * new_zoom as f64 / self.zoom as f64) as i64 + y_offset as i64;
        self.zoom = new_zoom;
        self.last_squares = HashMap::new();
    }

    pub fn move_center(&mut self, x: i64, y: i64) {
        self.center_x += x;
        self.center_y += y;
    }

    pub fn calculate_mandelbrot(&mut self) -> ImageBuffer<image::Rgb<u8>, Vec<u8>> {
        let square_size:u32 = 32;
        let mut imgbuf = ImageBuffer::new(self.width, self.height);
        let top_x = self.center_x - self.width as i64 / 2;
        let top_y = self.center_y - self.height as i64 / 2;
        let start_x = top_x - top_x % square_size as i64 - square_size as i64;
        let start_y = top_y - top_y % square_size as i64 - square_size as i64;

        let mut squares:Vec<Square> = Vec::new();
        for x in (start_x..top_x + self.width as i64).step_by(square_size as usize) {
            for y in (start_y..top_y + self.height as i64).step_by(square_size as usize) {
                squares.push(Square::new(x, y, self.zoom, square_size, self.max_iter));
            }
        }
        
        let square_results:Vec<(Square,SquareResult)> = squares.into_par_iter()
            .map(|square| {
                if let Some(sqr) = self.last_squares.get(&square) {
                    return (square, sqr.clone())
                }
                (square, square.calculate_square())
            })
            .collect();

        square_results.iter().for_each(|(_, square_result)| {
            for pixel in square_result.clone().into_iter() {
                if (pixel.x - top_x) >= self.width as i64 || (pixel.y - top_y) >= self.height as i64 || pixel.x < top_x || pixel.y < top_y {
                    continue
                }
                let x = (pixel.x - top_x) as u32;
                let y = (pixel.y - top_y) as u32;
                imgbuf.put_pixel(x, y, pixel.color);
            }
        });

        for (square, square_result) in square_results.into_iter() {
            self.last_squares.insert(square, square_result);
        }
        imgbuf
    } 
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
struct Square {
    x: i64,
    y: i64,
    zoom: u64,
    size: u32,
    max_iter: u32,
}

impl Square {
    pub fn new(x: i64, y: i64, zoom: u64, size: u32, max_iter: u32) -> Self {
        Square {
            x,
            y,
            zoom,
            size,
            max_iter,
        }
    }

    fn calculate_color(color: u32) -> Rgb<u8> {
        let num_colors = 160;
        if color == 0 {
            return Rgb([0,0,0]);
        } 
        let limited_input = (4.0 * (color as f64).powf(0.9)) as u32 % num_colors + 20 as u32;
        let hue = (limited_input as f32 / num_colors as f32) * 2.0 * std::f32::consts::PI;  
        let r = ((hue.sin() * 0.5 + 0.5) * 255.0) as u8;
        let g = ((hue.cos() * 0.5 + 0.5) * 255.0) as u8;
        let b = (((hue + std::f32::consts::PI / 2.0).cos() * 0.5 + 0.5) * 255.0) as u8;
        Rgb([r,g,b])
    }
    
    pub fn calculate_square(&self) -> SquareResult {
        let stepsize = 1.0 / self.zoom as f64;
        let prev = Complex::new(
            self.x as f64 * stepsize,
            self.y as f64 * stepsize,
        ).calculate_mandelbrot_iterations(self.max_iter);
        let mut all_equal = true;
        'outer: for a in (0..self.size as i64).step_by(2) {
            for b in [0,self.size as i64 - 1] {
                let c = Complex::new(
                    (self.x + a) as f64 * stepsize,
                    (self.y + b) as f64 * stepsize,
                );
                let result = c.calculate_mandelbrot_iterations(self.max_iter);
                if result != prev {
                    all_equal = false;
                    break 'outer;
                }                
                let c = Complex::new(
                    (self.x + b) as f64 * stepsize,
                    (self.y + a) as f64 * stepsize,
                );
                let result = c.calculate_mandelbrot_iterations(self.max_iter);
                if result != prev {
                    all_equal = false;
                    break 'outer;
                    
                }                
            }
        } 
        if all_equal {
            return SquareResult::new(vec![Self::calculate_color(prev); (self.size * self.size) as usize], self.x, self.y, self.size)
        }
        let mut colors = Vec::with_capacity((self.size * self.size) as usize);
        for y in 0..self.size as i64 {
            for x in 0..self.size as i64 {
                let c = Complex::new(
                    (self.x + x) as f64 * stepsize,
                    (self.y + y) as f64 * stepsize,
                );
                colors.push(Self::calculate_color(c.calculate_mandelbrot_iterations(self.max_iter)));
            }
        }
        SquareResult::new(colors, self.x, self.y, self.size)
    }
}

#[derive(Debug, Clone)]
struct SquareResult {
    colors: Vec<Rgb<u8>>,
    x: i64,
    y: i64,
    size: u32,
    index: usize,
}

impl SquareResult {
    pub fn new(colors: Vec<Rgb<u8>>, x: i64, y: i64, size: u32) -> Self {
        SquareResult {
            colors,
            x,
            y,
            size,
            index: 0,
        }
    }
}

impl Iterator for SquareResult {
    type Item = Pixel;
    
    fn next(&mut self) -> Option<Self::Item> {
        if self.index >= self.colors.len() {
            return None
        }
        let result = Pixel{ color: self.colors[self.index], x: self.x + (self.index as i64 % self.size as i64), y: self.y + (self.index as i64 / self.size as i64) } ;
        self.index += 1;
        Some(result)
    }

}

struct Pixel {
    x: i64,
    y: i64,
    color: Rgb<u8>,
}