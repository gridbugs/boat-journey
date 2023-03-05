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
use std::{
    collections::{HashSet, VecDeque},
    time::Duration,
};
use vector::{Cartesian, Radians};

pub mod witness;
mod world;

pub use gridbugs::entity_table::Entity;
pub use world::data::{Boat, Layer, Location, Tile};
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

enum MoveDirection {
    Forward,
    Backward,
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
        let mut game = Self {
            rng,
            world,
            player_entity,
        };
        let (boat_entity, boat) = game.world.components.boat.iter().next().unwrap();
        let boat_coord = game.world.spatial_table.coord_of(boat_entity).unwrap();
        if !game.try_rasterize_boat(boat_entity, boat.clone(), boat_coord) {
            panic!("failed to create the boat");
        }
        game
    }

    pub fn player_coord(&self) -> Coord {
        let (boat_enity, _boat) = self.world.components.boat.iter().next().unwrap();
        let boat_coord = self.world.spatial_table.coord_of(boat_enity).unwrap();
        boat_coord
    }

    fn try_rasterize_boat(&mut self, boat_entity: Entity, boat: Boat, boat_coord: Coord) -> bool {
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
                    .rotate_clockwise(boat.heading())
                    .to_cartesian()
                    .to_coord_round_nearest()
            })
            .collect::<Vec<_>>();
        let pairs = {
            let vs = &vertices_rotated;
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
        let mut boat_edge = HashSet::new();
        for (start, end) in pairs {
            for coord in coords_between_cardinal(start, end) {
                boat_edge.insert(coord);
            }
        }

        let mut boat_floor = HashSet::new();
        let mut to_visit = VecDeque::new();
        boat_floor.insert(Coord::new(0, 0));
        to_visit.push_back(Coord::new(0, 0));
        while let Some(coord) = to_visit.pop_front() {
            for d in CardinalDirection::all() {
                let nei_coord = coord + d.coord();
                if !boat_edge.contains(&nei_coord) {
                    if boat_floor.insert(nei_coord) {
                        to_visit.push_back(nei_coord);
                    }
                }
            }
        }

        for &coord in &boat_floor {
            if let Some(floor_entity) = self
                .world
                .spatial_table
                .layers_at_checked(coord + boat_coord)
                .floor
            {
                if !self.world.components.part_of_boat.contains(floor_entity) {
                    return false;
                }
            }
        }

        let mut edges_to_turn_into_floors = HashSet::new();
        for &coord in &boat_edge {
            if let Some(feature_entity) = self
                .world
                .spatial_table
                .layers_at_checked(coord + boat_coord)
                .feature
            {
                if !self.world.components.part_of_boat.contains(feature_entity) {
                    return false;
                }
            }
            if let Some(floor_entity) = self
                .world
                .spatial_table
                .layers_at_checked(coord + boat_coord)
                .floor
            {
                if !self.world.components.part_of_boat.contains(floor_entity) {
                    edges_to_turn_into_floors.insert(coord);
                }
            }
        }

        let mut to_delete = Vec::new();
        for entity in self.world.components.part_of_boat.entities() {
            to_delete.push(entity);
        }
        for entity in to_delete {
            self.world.components.remove_entity(entity);
            self.world.spatial_table.remove(entity);
        }
        for coord in edges_to_turn_into_floors {
            boat_edge.remove(&coord);
            self.world.spawn_board(coord + boat_coord);
        }
        for coord in boat_edge {
            self.world.spawn_boat_edge(coord + boat_coord);
        }
        for coord in boat_floor {
            self.world.spawn_boat_floor(coord + boat_coord);
        }
        self.world.components.boat.insert(boat_entity, boat);
        let _ = self
            .world
            .spatial_table
            .update_coord(boat_entity, boat_coord);
        let _ = self
            .world
            .spatial_table
            .update_coord(self.player_entity, boat_coord);

        true
    }

    pub fn render<F: FnMut(Location, Tile)>(&self, mut f: F) {
        self.world
            .components
            .tile
            .iter()
            .for_each(|(entity, &tile)| {
                if let Some(&location) = self.world.spatial_table.location_of(entity) {
                    f(location, tile);
                }
            });
    }

    fn rotate_boat(&mut self, rotate_direction: RotateDirection) {
        let (boat_entity, boat) = self.world.components.boat.iter().next().unwrap();
        let step_radians = std::f64::consts::FRAC_PI_4;
        let delta_radians = match rotate_direction {
            RotateDirection::Left => -step_radians,
            RotateDirection::Right => step_radians,
        };
        let boat_next = boat.add_heading(Radians(delta_radians));
        let boat_coord = self.world.spatial_table.coord_of(boat_entity).unwrap();
        self.try_rasterize_boat(boat_entity, boat_next, boat_coord);
    }

    fn move_boat(&mut self, move_direction: MoveDirection) {
        let (boat_entity, boat) = self.world.components.boat.iter().next().unwrap();
        let boat_coord = self.world.spatial_table.coord_of(boat_entity).unwrap();
        let (boat_next, delta) = match move_direction {
            MoveDirection::Forward => boat.step(),
            MoveDirection::Backward => boat.step_backwards(),
        };
        self.try_rasterize_boat(boat_entity, boat_next, boat_coord + delta);
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
            Input::Walk(CardinalDirection::North) => self.move_boat(MoveDirection::Forward),
            Input::Walk(CardinalDirection::South) => self.move_boat(MoveDirection::Backward),
            _ => (),
        }
        Ok(None)
    }
}
