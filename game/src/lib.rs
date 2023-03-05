pub use gridbugs::{
    direction::CardinalDirection,
    entity_table::ComponentTable,
    grid_2d::{Coord, Grid, Size},
    line_2d::{coords_between, coords_between_cardinal},
    shadowcast::Context as ShadowcastContext,
};
use rand::{Rng, SeedableRng};
use rand_isaac::Isaac64Rng;
use serde::{Deserialize, Serialize};
use std::time::Duration;
use vector::{Cartesian, Radial, Radians};

pub mod witness;
mod world;

pub use gridbugs::entity_table::Entity;
pub use world::data::Tile;
use world::World;

mod terrain;
use terrain::Terrain;

pub const MAP_SIZE: Size = Size::new_u16(20, 14);

#[derive(Debug, Clone, Copy)]
pub struct Omniscient;

#[derive(Debug, Clone, Copy)]
pub struct Config {
    pub omniscient: Option<Omniscient>,
    pub demo: bool,
    pub debug: bool,
}
impl Config {
    pub const OMNISCIENT: Option<Omniscient> = Some(Omniscient);
}
impl Default for Config {
    fn default() -> Self {
        Self {
            omniscient: None,
            demo: false,
            debug: false,
        }
    }
}

#[derive(Debug)]
pub enum GameControlFlow {
    GameOver,
    Win,
}

#[derive(Clone, Copy, Debug)]
pub enum Input {
    Walk(CardinalDirection),
    Wait,
}

enum RotateDirection {
    Left,
    Right,
}

#[derive(Serialize, Deserialize)]
pub struct Game {
    world: World,
    rng: Isaac64Rng,
    player_entity: Entity,
}

pub enum ActionError {}

impl Game {
    pub fn new<R: Rng>(_config: &Config, base_rng: &mut R) -> Self {
        let mut rng = Isaac64Rng::seed_from_u64(base_rng.gen());
        let Terrain {
            world,
            player_entity,
        } = Terrain::generate(world::spawn::make_player(), &mut rng);
        let game = Self {
            rng,
            world,
            player_entity,
        };
        game
    }

    pub fn render<F: FnMut(Coord, Tile)>(&self, mut f: F) {
        let (boat_enity, boat) = self.world.components.boat.iter().next().unwrap();
        let boat_coord = self.world.spatial_table.coord_of(boat_enity).unwrap();

        let boat_width1 = 3;
        let boat_width2 = 5;
        let boat_length1 = 3;
        let boat_length2 = 7;
        let boat_length3 = 7;
        let boat_length4 = 10;
        let vertices_int = vec![
            Coord::new(0, -boat_length4),            // 0
            Coord::new(boat_width1, -boat_length3),  // 1
            Coord::new(boat_width2, boat_length1),   // 2
            Coord::new(boat_width1, boat_length2),   // 3
            Coord::new(0, boat_length2),             // 4
            Coord::new(-boat_width1, boat_length2),  // 5
            Coord::new(-boat_width2, boat_length1),  // 6
            Coord::new(-boat_width1, -boat_length3), // 7
        ];
        let vertices_rotated = vertices_int
            .into_iter()
            .map(|v| {
                Cartesian::from_coord(v)
                    .to_radial()
                    .rotate_clockwise(boat.heading)
                    .to_cartesian()
                    .to_coord_round_nearest()
                    + boat_coord
            })
            .collect::<Vec<_>>();

        let pairs = {
            let vs = vertices_rotated;
            vec![
                (vs[0], vs[1]),
                (vs[1], vs[2]),
                (vs[2], vs[3]),
                (vs[3], vs[4]),
                (vs[0], vs[7]),
                (vs[7], vs[6]),
                (vs[6], vs[5]),
                (vs[5], vs[4]),
            ]
        };

        for (start, end) in pairs.into_iter() {
            for coord in coords_between_cardinal(start, end) {
                f(coord, Tile::BoatEdge);
            }
        }
    }

    fn rotate_boat(&mut self, rotate_direction: RotateDirection) {
        let (_, boat) = self.world.components.boat.iter_mut().next().unwrap();
        let step_radians = std::f64::consts::FRAC_PI_4;
        let delta_radians = match rotate_direction {
            RotateDirection::Left => -step_radians,
            RotateDirection::Right => step_radians,
        };
        boat.heading.0 += delta_radians;
    }

    #[must_use]
    pub(crate) fn handle_tick(
        &mut self,
        _since_last_tick: Duration,
        _config: &Config,
    ) -> Option<GameControlFlow> {
        None
    }

    #[must_use]
    pub(crate) fn handle_input(
        &mut self,
        input: Input,
        _config: &Config,
    ) -> Result<Option<GameControlFlow>, ActionError> {
        match input {
            Input::Walk(CardinalDirection::East) => self.rotate_boat(RotateDirection::Right),
            Input::Walk(CardinalDirection::West) => self.rotate_boat(RotateDirection::Left),
            _ => (),
        }
        Ok(None)
    }
}
