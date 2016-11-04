extern crate rand;
extern crate framebuffer;

use rand::Rng;
use framebuffer::Framebuffer;

const DUST_SPEED: f32 = 1.003;
const DUST_GROWTH: f32 = 1.002;
const DUST_COUNT: usize = 40000;

struct Dustfield {
    dusts: [Dust; DUST_COUNT],
    scene: DustfieldScene,
    screen: (u32, u32, u32),
}

enum DustfieldScene {
    Brownian(f32),
    Movement(f32, f32),
    Blackhole(f32, f32),
    Whitehole(f32, f32),
    Gravity(f32),
}

impl Dustfield {

    fn new(framebuffer: &Framebuffer) -> Dustfield {
        let (w, h) = framebuffer.resolution();
        let bpp = framebuffer.bits_per_pixel();

        let mut dusts = [Dust::new(); DUST_COUNT];
        for dust in dusts.iter_mut() {
            dust.init(w as usize, h as usize);
        }
        Dustfield {
            dusts: dusts,
            scene: DustfieldScene::Brownian(1.0),
            screen: ( w, h, bpp ),
        }
    }
    
    fn tick(&mut self, frame: &mut [u32]) {
        let (w, h, bpp) = self.screen;
        let mut rng = rand::thread_rng();
        // change scene
        if rng.gen_weighted_bool(100) {
            if rng.gen_weighted_bool(10) {
                let x = rng.gen_range::<f32>(0.0, w as f32);
                let y = rng.gen_range::<f32>(0.0, h as f32);
                self.scene = DustfieldScene::Blackhole(x,y);
                println!("Spawned blackhole at {},{}", x, y);
            }else if rng.gen_weighted_bool(11) {
                let x = rng.gen_range::<f32>(0.0, w as f32);
                let y = rng.gen_range::<f32>(0.0, h as f32);
                self.scene = DustfieldScene::Whitehole(x,y);
                println!("Spawned whitehole at {},{}", x, y);
            }else if rng.gen_weighted_bool(5) {
                let x = rng.gen_range::<f32>(-7.0, 7.0);
                let y = rng.gen_range::<f32>(-7.0, 7.0);
                self.scene = DustfieldScene::Movement(x,y);
                println!("Movement: {},{}", x, y);
            }else if rng.gen_weighted_bool(3) {
                let y = rng.gen_range::<f32>(0.0, 30.0);
                self.scene = DustfieldScene::Gravity(y);
                println!("Gravity: {}", y);
            }else{
                let x = rng.gen_range::<f32>(0.0, 3.0);
                self.scene = DustfieldScene::Brownian(x);
                println!("Back to brownian motion: {}", x);
            }
        }

        for dust in self.dusts.iter_mut() {
            let dust_data = dust.tick(w as usize, h as usize, &self.scene);
            if dust_data.0 < w as usize && dust_data.1 < h as usize { 
                Dustfield::draw_dust(dust_data, self.screen, frame);
            }
        }
    }


    fn draw_dust(dust_data: (usize, usize, f32), screen: (u32, u32, u32), frame: &mut[u32]) {
        let (w, h, bpp) = screen;
    
        let bytespp = 1;
        let line_length = bytespp * w as usize;

        macro_rules! coords_to_index {
            ($x:expr, $y: expr) => { $y * line_length + $x * bytespp }
        }

        let dim = dust_data.2 as usize;
        for i in 0 .. dim {
            for j in 0 .. dim {
                if dust_data.0 + i < w as usize && dust_data.1 + j < h as usize {

                    frame[coords_to_index!(dust_data.0 + i, dust_data.1 + j)] = 0x77;
         //           frame[coords_to_index!(dust_data.0 + i, dust_data.1 + j) + 1] = 0x00;
         //           frame[coords_to_index!(dust_data.0 + i, dust_data.1 + j) + 2] = 0x00;
                }
            }
        }
    }
}

#[derive(Clone, Copy)]
struct Dust {
    x: f32,
    y: f32,
    z: f32,
    max_speed: f32,
    color: u32,
}

impl Dust {

    fn new() -> Dust {
        Dust { 
            x: 0.0, 
            y: 0.0,
            z: 0.0,
            max_speed: 0.0,
            color: 0,
        }
    }

    fn init(&mut self, w: usize, h: usize) {
        let mut rng = rand::thread_rng();
        self.x = rng.gen_range::<f32>(0.0, w as f32);
        self.y = rng.gen_range::<f32>(0.0, h as f32);
        self.z = rng.gen_range::<f32>(2.0, 4.0);
        self.max_speed = 15.0;
        self.color = 0x000077; //dark blue
    }

    fn get_pos(&self, w: usize, h: usize) -> (usize, usize) {
        (self.x as usize, self.y as usize)
    }

    fn get_color(&self) -> u32 {
        self.color
    }

    fn tick(&mut self, w: usize, h: usize, scene: &DustfieldScene) -> (usize, usize, f32) {
        let mut rng = rand::thread_rng();
        let x1 = rng.gen_range::<f32>(-self.max_speed/2.0, self.max_speed/2.0);
        let y1 = rng.gen_range::<f32>(-self.max_speed/2.0, self.max_speed/2.0);

        match *scene {
            DustfieldScene::Brownian(s) => {
                self.x += x1*s;
                self.y += y1*s;
            },
            DustfieldScene::Blackhole(xb, yb) => {
                let speed2 = -self.max_speed * self.max_speed / 
                    ((self.x - xb)*(self.x - xb) + (self.y - yb)*(self.y - yb)); 
                self.x += speed2*(self.x - xb);
                self.y += speed2*(self.y - yb);
                self.x += x1;
                self.y += y1;
            },
            DustfieldScene::Whitehole(xb, yb) => {
                let speed2 = self.max_speed * self.max_speed / 
                    ((self.x - xb)*(self.x - xb) + (self.y - yb)*(self.y - yb)); 
                self.x += speed2*(self.x - xb);
                self.y += speed2*(self.y - yb);
                self.x += x1;
                self.y += y1;
            },
            DustfieldScene::Gravity(g) => {
                self.y += g*self.z*self.z*self.z/100.0;
                self.x += x1;
            },
            DustfieldScene::Movement(xb, yb) => {
                self.x += xb;
                self.y += yb;
            },

            //_ => { },
        }

        if self.x >= w as f32 {
            self.x -= w as f32;
        }
        if self.y >= h as f32 {
            self.y -= h as f32;
        }
        if self.x < 0.0 {
            self.x += w as f32;
        }
        if self.y < 0.0 {
            self.y += h as f32;
        }

        let pos = self.get_pos(w, h);

        (pos.0, pos.1, self.z)
    }
}

fn main() {
    let mut framebuffer = Framebuffer::new("/dev/fb0").unwrap();

    let (w, h) = framebuffer.resolution();
    // in this example we write one pixel at a time
    let mut frame = vec![0u32; w as usize * h as usize];

    let mut field = Dustfield::new(&framebuffer);
    let mut i = 0;

    loop {
        for x in frame.iter_mut() { *x = 0; }
        field.tick(&mut frame);

        let _ = framebuffer.write_frame(&frame);
        i += 1;
    }
}
