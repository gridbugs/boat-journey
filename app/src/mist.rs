use coord_2d::Coord;
use perlin2::Perlin2;
use rand::Rng;
use rgb_int::Rgba32;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Mist {
    perlin: Perlin2,
    intensity: f64,
    offset_x: f64,
    speed_x: f64,
}

impl Mist {
    pub fn new<R: Rng>(rng: &mut R) -> Self {
        Self {
            perlin: Perlin2::new(rng),
            intensity: 0.03,
            offset_x: 0.,
            speed_x: 0.005,
        }
    }

    pub fn get(&self, coord: Coord) -> Rgba32 {
        let noise = self
            .perlin
            .noise01((coord.x as f64 * 0.05 + self.offset_x, coord.y as f64 * 0.2));
        let alpha = (self.intensity * 255. * noise) as u8;
        Rgba32::new(255, 255, 255, alpha)
    }

    pub fn tick(&mut self) {
        self.offset_x += self.speed_x;
    }
}
