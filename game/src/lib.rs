pub use gridbugs::{
    direction::{CardinalDirection, Direction},
    entity_table::ComponentTable,
    grid_2d::{Coord, Grid, Size},
    line_2d::{coords_between, coords_between_cardinal},
    rgb_int::Rgb24,
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

pub use gridbugs::{
    entity_table::{entity_data, entity_update, Entity},
    visible_area_detection::{
        vision_distance::Circle, CellVisibility, VisibilityGrid, World as VisibleWorld,
    },
};
pub use world::data::{Boat, Layer, Location, Tile};
use world::{
    data::{DoorState, EntityData, EntityUpdate},
    spatial::{LayerTable, Layers},
    World,
};

mod terrain;
use terrain::Terrain;

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
    DriveToggle,
}

enum RotateDirection {
    Left,
    Right,
}

enum MoveDirection {
    Forward,
    Backward,
}

#[derive(Serialize, Deserialize, Default)]
pub struct VisibleCellData {
    pub tiles: LayerTable<Option<Tile>>,
}

impl VisibleCellData {
    fn update(&mut self, world: &World, coord: Coord) {
        let layers = world.spatial_table.layers_at_checked(coord);
        self.tiles = layers.option_and_then(|&entity| world.components.tile.get(entity).cloned());
    }
}

impl VisibleWorld for World {
    type VisionDistance = Circle;

    fn size(&self) -> Size {
        self.spatial_table.grid_size()
    }

    fn get_opacity(&self, coord: Coord) -> u8 {
        if let Some(&Layers {
            feature: Some(feature_entity),
            ..
        }) = self.spatial_table.layers_at(coord)
        {
            self.components
                .opacity
                .get(feature_entity)
                .cloned()
                .unwrap_or(0)
        } else {
            0
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct Game {
    world: World,
    rng: Isaac64Rng,
    player_entity: Entity,
    driving: bool,
    visibility_grid: VisibilityGrid<VisibleCellData>,
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
            visibility_grid: VisibilityGrid::new(world.spatial_table.grid_size()),
            world,
            player_entity,
            driving: true,
        };
        let (boat_entity, boat) = game.world.components.boat.iter().next().unwrap();
        let boat_coord = game.world.spatial_table.coord_of(boat_entity).unwrap();
        if !game.try_rasterize_boat(boat_entity, boat.clone(), boat_coord) {
            panic!("failed to create the boat");
        }
        game.update_visibility();
        game
    }

    pub fn update_visibility(&mut self) {
        let update_fn = |data: &mut VisibleCellData, coord| data.update(&self.world, coord);
        self.visibility_grid.update_custom(
            Rgb24::new_grey(255),
            &self.world,
            Circle::new_squared(500),
            self.player_coord(),
            update_fn,
        );
    }

    fn try_rasterize_boat(&mut self, boat_entity: Entity, boat: Boat, boat_coord: Coord) -> bool {
        //        #
        //      #####
        //     ##   ##
        //    ##     ##
        //    #   @   #
        //    #       #
        //    # ###+# #
        //    # #   # #
        //    # #   # #
        //    # #+### #
        //    #       #
        //    ##     ##
        //     #######

        let boat_width1 = 4;
        let boat_width2 = 4;
        let boat_width3 = 3;
        let boat_length1 = 8;
        let boat_length2 = 10;
        let boat_length3 = 1;
        let boat_length4 = 6;
        let vertices_int = vec![
            Coord::new(0, -boat_length4),            // 0
            Coord::new(boat_width1, -boat_length3),  // 1
            Coord::new(boat_width2, boat_length1),   // 2
            Coord::new(boat_width3, boat_length2),   // 3
            Coord::new(0, boat_length2),             // 4
            Coord::new(-boat_width3, boat_length2),  // 5
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

        let boat_heading = boat.heading();

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
        boat_floor.remove(&Coord::new(0, 0));
        for coord in boat_floor {
            self.world.spawn_boat_floor(coord + boat_coord);
        }
        self.world.spawn_boat_controls(boat_coord);
        self.world.components.boat.insert(boat_entity, boat);
        let _ = self
            .world
            .spatial_table
            .update_coord(boat_entity, boat_coord);
        let _ = self
            .world
            .spatial_table
            .update_coord(self.player_entity, boat_coord);

        if false {
            // the cabin is tricky to get right when the boat is not facing a cardinal direction
            let unit = Cartesian { x: 1., y: 0. }
                .to_radial()
                .rotate_clockwise(boat_heading)
                .to_cartesian()
                .to_coord_round_nearest();
            let is_cardinal = unit.x * unit.y == 0;

            if is_cardinal {
                let cabin_top_left = Cartesian { x: -2., y: 3. }
                    .to_radial()
                    .rotate_clockwise(boat_heading)
                    .to_cartesian()
                    .to_coord_round_nearest();
                let cabin_top_right = Cartesian { x: 0., y: 3. }
                    .to_radial()
                    .rotate_clockwise(boat_heading)
                    .to_cartesian()
                    .to_coord_round_nearest();
                let cabin_bottom_left = Cartesian { x: -2., y: 7. }
                    .to_radial()
                    .rotate_clockwise(boat_heading)
                    .to_cartesian()
                    .to_coord_round_nearest();
                for coord in coords_between_cardinal(cabin_top_left, cabin_top_right) {
                    self.world.spawn_boat_wall(coord + boat_coord);
                }
                for coord in coords_between_cardinal(cabin_top_left, cabin_bottom_left).skip(1) {
                    self.world.spawn_boat_wall(coord + boat_coord);
                }
                let cabin_top_right = Cartesian { x: 2., y: 3. }
                    .to_radial()
                    .rotate_clockwise(boat_heading)
                    .to_cartesian()
                    .to_coord_round_nearest();
                let cabin_bottom_right = Cartesian { x: 2., y: 7. }
                    .to_radial()
                    .rotate_clockwise(boat_heading)
                    .to_cartesian()
                    .to_coord_round_nearest();
                let cabin_bottom_left = Cartesian { x: 0., y: 7. }
                    .to_radial()
                    .rotate_clockwise(boat_heading)
                    .to_cartesian()
                    .to_coord_round_nearest();
                for coord in coords_between_cardinal(cabin_top_right, cabin_bottom_right) {
                    self.world.spawn_boat_wall(coord + boat_coord);
                }
                for coord in coords_between_cardinal(cabin_bottom_right, cabin_bottom_left).skip(1)
                {
                    self.world.spawn_boat_wall(coord + boat_coord);
                }
            }
        }

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

    pub fn cell_visibility_at_coord(&self, coord: Coord) -> CellVisibility<&VisibleCellData> {
        self.visibility_grid.get_visibility(coord)
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

    // Returns the coordinate of the player character
    pub fn player_coord(&self) -> Coord {
        self.world
            .spatial_table
            .coord_of(self.player_entity)
            .expect("player does not have coord")
    }

    fn open_door(&mut self, entity: Entity) {
        self.world.components.apply_entity_update(
            entity,
            entity_update! {
                door_state: Some(DoorState::Open),
                tile: Some(Tile::DoorOpen),
                solid: None,
                opacity: None,
            },
        );
    }

    fn open_door_entity_adjacent_to_coord(&self, coord: Coord) -> Option<Entity> {
        for direction in Direction::all() {
            let potential_door_coord = coord + direction.coord();
            if let Some(&Layers {
                feature: Some(feature_entity),
                ..
            }) = self.world.spatial_table.layers_at(potential_door_coord)
            {
                if let Some(DoorState::Open) = self.world.components.door_state.get(feature_entity)
                {
                    return Some(feature_entity);
                }
            }
        }
        None
    }

    fn close_door(&mut self, entity: Entity) {
        self.world.components.insert_entity_data(
            entity,
            entity_data! {
                door_state: DoorState::Closed,
                tile: Tile::DoorClosed,
                solid: (),
                opacity: 255,
            },
        );
    }

    fn player_walk(&mut self, direction: CardinalDirection) {
        let player_coord = self.player_coord();
        let new_player_coord = player_coord + direction.coord();
        let layers = self.world.spatial_table.layers_at(new_player_coord);
        if let Some(&Layers {
            feature: Some(feature_entity),
            ..
        }) = layers
        {
            // If the player bumps into a door, open the door
            if let Some(DoorState::Closed) = self.world.components.door_state.get(feature_entity) {
                self.open_door(feature_entity);
                return;
            }
            // Don't let the player walk through solid entities
            if self.world.components.solid.contains(feature_entity) {
                if let Some(open_door_entity) =
                    self.open_door_entity_adjacent_to_coord(player_coord)
                {
                    self.close_door(open_door_entity);
                }
                return;
            }
        }
        if let Some(&Layers {
            water: Some(_),
            floor: None,
            feature: None,
            ..
        }) = layers
        {
            return;
        }
        self.world
            .spatial_table
            .update_coord(self.player_entity, new_player_coord)
            .unwrap();
    }

    fn is_player_on_driving_coord(&self) -> bool {
        let player_coord = self.player_coord();
        let (boat_entity, _boat) = self.world.components.boat.iter().next().unwrap();
        let boat_coord = self.world.spatial_table.coord_of(boat_entity).unwrap();
        player_coord == boat_coord
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
        if self.driving {
            match input {
                Input::Walk(CardinalDirection::East) => self.rotate_boat(RotateDirection::Right),
                Input::Walk(CardinalDirection::West) => self.rotate_boat(RotateDirection::Left),
                Input::Walk(CardinalDirection::North) => self.move_boat(MoveDirection::Forward),
                Input::Walk(CardinalDirection::South) => self.move_boat(MoveDirection::Backward),
                Input::DriveToggle => self.driving = false,
                _ => (),
            }
        } else {
            match input {
                Input::Walk(direction) => self.player_walk(direction),
                Input::DriveToggle => {
                    if self.is_player_on_driving_coord() {
                        self.driving = true;
                    }
                }
                _ => (),
            }
        }
        self.update_visibility();
        Ok(None)
    }
}
