extern crate framebuffer;
extern crate time;

use framebuffer::Framebuffer;
use time::PreciseTime;

struct VerticalBars {
    x0: u32,
    separation: u32,
    width: u32,
    speed: u32,
}

impl VerticalBars {
    fn new() -> VerticalBars {
        VerticalBars {
            x0: 0,
            separation: 60,
            width: 40,
            speed: 10,
        }
    }

    fn tick(&mut self, res: (u32, u32), frame: &mut [u8]) {
        let (w, h) = res;
        self.x0 += self.speed;
        while self.x0 >= self.separation + self.width {
            self.x0 -= self.separation + self.width;
        }

        let mut x = self.x0;
        while x < w {
            VerticalBars::draw_bar((x, x+self.width), (w, h) , frame);
            x += self.separation + self.width;
        }
    }


    fn draw_bar(data: (u32, u32), res: (u32, u32), frame: &mut[u8]) {
        let (w, h) = res;
        // assuming 32 bpp
        let bytes_per_pixel = 4;
        let line_length = w * 4;

        macro_rules! coords_to_index {
            ($x:expr, $y: expr) => { $y * line_length as usize + $x * bytes_per_pixel }
        }

        for y in 0..h {
            for x in data.0..data.1 {
                if x < w {
                    frame[coords_to_index!(x as usize, y as usize)] = 0xFF;
                    frame[coords_to_index!(x as usize, y as usize) + 1] = 0xFF;
                    frame[coords_to_index!(x as usize, y as usize) + 2] = 0xFF;
                }
            }
        }
    }
}

fn main() {
    let mut framebuffer = Framebuffer::new("/dev/fb0").unwrap();

    let (w, h) = framebuffer.resolution();

    let mut bars = VerticalBars::new();

    let mut i = 0;
    let mut start = PreciseTime::now();
    loop {
        {
            let mut frame : &mut [u8] = framebuffer.frame_mut();
            for x in frame.iter_mut() { *x = 0; }
            bars.tick((w,h), &mut frame);
        }

        let end = PreciseTime::now();
        let duration = start.to(end).num_microseconds().unwrap();
        start = PreciseTime::now();
        if i % 10 == 0 {
            println!("{} milliseconds: {} fps", duration as f64 / 1000.0, (1.0)/(duration as f64)*1000000.0f64);
        }
        i += 1;
    }
}
