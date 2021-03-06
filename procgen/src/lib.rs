use grid_2d::{coord_2d::Axis, Grid, Size};
use rand::Rng;

mod doors;
mod hull;
mod internal_walls;
mod windows;

pub enum GameCell {
    Wall,
    Floor,
    Space,
    Door(Axis),
    Window(Axis),
    Stairs,
    Spawn,
}

#[derive(Debug)]
pub struct Spec {
    pub size: Size,
}

pub fn generate<R: Rng>(spec: Spec, rng: &mut R) -> Grid<GameCell> {
    loop {
        let hull = hull::generate_hull(spec.size, rng);
        let with_internal_walls = internal_walls::add_internal_walls(&hull, rng);
        if let Some(with_doors) = doors::add_doors(&with_internal_walls, rng) {
            let with_windows = windows::add_windows(&with_doors, rng);
            return Grid::new_fn(spec.size, |coord| {
                use windows::WindowCell;
                match with_windows.get_checked(coord) {
                    WindowCell::Wall => GameCell::Wall,
                    WindowCell::Floor => GameCell::Floor,
                    WindowCell::Space => GameCell::Space,
                    WindowCell::Door(axis) => GameCell::Door(*axis),
                    WindowCell::Window(axis) => GameCell::Window(*axis),
                    WindowCell::Stairs => GameCell::Stairs,
                    WindowCell::Spawn => GameCell::Spawn,
                }
            });
        }
    }
}
