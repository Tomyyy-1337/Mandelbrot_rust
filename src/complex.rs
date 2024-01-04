#[derive(Debug, Clone, Copy)]
pub struct Complex {
    pub real: f64,
    pub imag: f64,
}

impl Complex {
    pub fn new(real: f64, imag: f64) -> Self {
        Self { real, imag }
    }

    fn square(&self) -> Self {
        Self {
            real: (self.real + self.imag) * (self.real - self.imag),
            imag: 2.0 * self.real * self.imag,
        }
    }

    fn add(&self, other: &Complex) -> Self {
        Self {
            real: self.real + other.real,
            imag: self.imag + other.imag,
        }
    }

    fn norm(&self) -> f64 {
        self.real * self.real + self.imag * self.imag
    }

    pub fn calculate_mandelbrot_iterations(&self, max_iter: u32) -> u32 {
        let mut z = Self::new(0.0, 0.0);
        for counter in 1..=max_iter {
            z = z.square().add(self);
            if z.norm() >= 4.0 {
                return counter;
            }
        }
        0
    }
}