pub use gridbugs::{
    direction::{CardinalDirection, Direction},
    entity_table::ComponentTable,
    grid_2d::{Coord, Grid, Size},
    grid_search_cardinal::distance_map,
    line_2d::{coords_between, coords_between_cardinal},
    rgb_int::Rgb24,
    shadowcast::Context as ShadowcastContext,
    spatial_table::UpdateError,
};
use rand::{seq::SliceRandom, Rng, SeedableRng};
use rand_isaac::Isaac64Rng;
use serde::{Deserialize, Serialize};
use std::{
    collections::{HashSet, VecDeque},
    time::Duration,
};
use vector::{Cartesian, Radial, Radians};

pub mod witness;
mod world;

pub use gridbugs::{
    entity_table::{entity_data, entity_update, Entity},
    line_2d,
    visible_area_detection::{
        vision_distance::Circle, CellVisibility, VisibilityGrid, World as VisibleWorld,
    },
};
pub use world::data::{Boat, Layer, Location, Meter, Npc, Tile};
use world::{
    data::{Components, DoorState, EntityData, EntityUpdate},
    spatial::{LayerTable, Layers, SpatialTable},
    World,
};

mod terrain;
use terrain::{Dungeon, Terrain};

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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Victory {
    pub name: String,
    pub stats: VictoryStats,
}

impl Victory {
    fn text(&self) -> String {
        format!(
            "Here lies {}.\n\n\nThey reached the ocean after {} turns over {} days.",
            self.name, self.stats.num_turns, self.stats.num_days
        )
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VictoryStats {
    pub num_turns: u64,
    pub num_days: u64,
}

impl VictoryStats {
    pub fn new() -> Self {
        Self {
            num_turns: 0,
            num_days: 1,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum GameOverReason {
    OutOfFuel,
    KilledByGhost,
    KilledByBeast,
    Abandoned,
    KilledBySoldier,
}

#[derive(Debug, Clone)]
pub enum MenuChoice {
    SayNothing,
    Leave,
    AddNpcToPassengers(Entity),
    DontAddNpcToPassengers,
    BuyFuel { cost: u32, amount: u32 },
    BuyCrewCapacity(u32),
    SleepUntilMorning(u32),
    StayAtInnForever,
    AbandonQuest,
    ChangeMind,
    Okay,
}

#[derive(Debug, Clone, Copy)]
pub enum MenuImage {
    Townsperson,
    Grave,
    Shop,
    Npc(Npc),
}

#[derive(Debug, Clone)]
pub struct Menu {
    pub choices: Vec<MenuChoice>,
    pub text: String,
    pub image: MenuImage,
}

#[derive(Debug)]
pub enum GameControlFlow {
    GameOver(GameOverReason),
    Win,
    Menu(Menu),
    Aim(Npc),
}

#[derive(Clone, Copy, Debug)]
pub enum Input {
    Walk(CardinalDirection),
    Wait,
    DriveToggle,
    Ability(u8),
}

enum RotateDirection {
    Left,
    Right,
}

enum MoveDirection {
    Forward,
    Backward,
}

#[derive(Serialize, Deserialize, Default, Debug)]
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
struct DungeonState {
    world_tmp: World,
    dungeon_index: usize,
    return_coord: Coord,
    dungeon_spawn: Coord,
    visibility_grid: VisibilityGrid<VisibleCellData>,
}

#[derive(Serialize, Deserialize, Default)]
struct AiCtx {
    distance_map: distance_map::PopulateContext,
}

#[derive(Serialize, Deserialize)]
pub struct Game {
    world: World,
    rng: Isaac64Rng,
    player_entity: Entity,
    driving: bool,
    visibility_grid: VisibilityGrid<VisibleCellData>,
    dungeons: Vec<Option<Dungeon>>,
    dungeon_state: Option<DungeonState>,
    stats: Stats,
    has_been_on_boat: bool,
    has_crossed_threshold: bool,
    has_talked_to_npc: bool,
    night_turn_count: u32,
    messages: Vec<String>,
    victory_stats: VictoryStats,
    passengers: Vec<Npc>,
    num_seats: u32,
    seat_rng_seed: u64,
    ai_ctx: AiCtx,
}

#[derive(Serialize, Deserialize)]
pub struct Stats {
    pub health: Meter,
    pub fuel: Meter,
    pub day: Meter,
    pub junk: Meter,
}

impl Stats {
    fn new() -> Self {
        let day_max = 1000;
        let first_day_skip = 100;
        Self {
            health: Meter::new(4, 4),
            fuel: Meter::new(400, 800),
            day: Meter::new(day_max - first_day_skip, day_max),
            junk: Meter::new(0, 10),
        }
    }
}

pub enum ActionError {}

impl Game {
    pub fn new<R: Rng>(_config: &Config, victories: Vec<Victory>, base_rng: &mut R) -> Self {
        let mut rng = Isaac64Rng::seed_from_u64(base_rng.gen());
        let Terrain {
            world,
            player_entity,
            num_dungeons,
        } = Terrain::generate(world::spawn::make_player(), victories, &mut rng);
        let dungeons = (0..=num_dungeons)
            .map(|_| Some(Dungeon::generate(&mut rng)))
            .collect::<Vec<_>>();
        let mut game = Self {
            seat_rng_seed: rng.gen(),
            rng,
            visibility_grid: VisibilityGrid::new(world.spatial_table.grid_size()),
            world,
            player_entity,
            driving: false,
            dungeons,
            dungeon_state: None,
            stats: Stats::new(),
            has_been_on_boat: false,
            has_crossed_threshold: false,
            has_talked_to_npc: false,
            night_turn_count: 0,
            messages: Vec::new(),
            victory_stats: VictoryStats::new(),
            passengers: vec![Npc::Soldier, Npc::Physicist],
            num_seats: 2,
            ai_ctx: Default::default(),
        };
        let (boat_entity, boat) = game.world.components.boat.iter().next().unwrap();
        let boat_coord = game.world.spatial_table.coord_of(boat_entity).unwrap();
        if !game.try_rasterize_boat(boat_entity, boat.clone(), boat_coord) {
            panic!("failed to create the boat");
        }
        game.update_visibility();
        if game.player_coord() == boat_coord {
            game.driving = true;
        }
        game
    }

    pub fn current_day(&self) -> u32 {
        self.victory_stats.num_days as u32
    }

    fn start_day(&mut self) {
        self.victory_stats.num_days += 1;
        self.stats.day.fill();
        self.stats.health.fill();
        self.night_turn_count = 0;
        self.messages.clear();
        let ghosts = self.world.components.ghost.entities().collect::<Vec<_>>();
        for entity in ghosts {
            self.world.components.remove_entity(entity);
            self.world.spatial_table.remove(entity);
        }
    }

    pub fn num_seats(&self) -> u32 {
        self.num_seats
    }

    pub fn passengers(&self) -> &[Npc] {
        &self.passengers
    }

    fn num_empty_seats(&self) -> u32 {
        self.num_seats - self.passengers.len() as u32
    }

    pub fn victory_stats(&self) -> &VictoryStats {
        &self.victory_stats
    }

    pub fn spawn_ghost(&mut self) {
        let angle = Radians(self.rng.gen::<f64>() * (2.0 * std::f64::consts::PI));
        let length = 10.;
        let coord = Radial { length, angle }
            .to_cartesian()
            .to_coord_round_nearest()
            + self.player_coord();
        if !self.is_coord_inside(coord) {
            self.world.spawn_ghost(coord);
        }
    }

    pub fn is_player_outside_at_night(&self) -> bool {
        !self.is_in_dungeon() && self.stats.day.is_empty() && !self.is_player_inside()
    }

    pub fn pass_time(&mut self) {
        self.victory_stats.num_turns += 1;
        if self.has_been_on_boat {
            self.stats.day.decrease(1);
        }
        if self.is_player_outside_at_night() {
            if self.night_turn_count % 20 == 0 {
                self.spawn_ghost();
            }
            self.night_turn_count += 1;
            let (boat_entity, boat) = self.world.components.boat.iter().next().unwrap();
            let boat_coord = self.world.spatial_table.coord_of(boat_entity).unwrap();
            if !self.try_rasterize_boat(boat_entity, boat.clone(), boat_coord) {
                panic!("failed to create the boat");
            }
        } else {
            self.night_turn_count = 0;
        }
    }

    pub fn spend_fuel(&mut self) {
        self.stats.fuel.decrease(1)
    }

    fn take_damage(&mut self) {
        self.stats.health.decrease(1)
    }

    pub fn messages(&self) -> &[String] {
        &self.messages
    }

    pub fn is_coord_inside(&self, coord: Coord) -> bool {
        if let Layers {
            floor: Some(floor), ..
        } = self.world.spatial_table.layers_at_checked(coord)
        {
            if self.world.components.inside.contains(*floor) {
                return true;
            }
        }
        false
    }

    pub fn is_player_inside(&self) -> bool {
        self.is_coord_inside(self.player_coord())
    }

    pub fn is_player_on_boat(&self) -> bool {
        if let Layers { boat: Some(_), .. } = self
            .world
            .spatial_table
            .layers_at_checked(self.player_coord())
        {
            true
        } else {
            if let Layers {
                floor: Some(floor), ..
            } = self
                .world
                .spatial_table
                .layers_at_checked(self.player_coord())
            {
                self.world.components.part_of_boat.contains(*floor)
            } else {
                false
            }
        }
    }

    pub fn stats(&self) -> &Stats {
        &self.stats
    }

    pub fn has_been_on_boat(&self) -> bool {
        self.has_been_on_boat
    }
    pub fn has_crossed_threshold(&self) -> bool {
        self.has_crossed_threshold
    }
    pub fn has_talked_to_npc(&self) -> bool {
        self.has_talked_to_npc
    }

    pub fn is_driving(&self) -> bool {
        self.driving
    }

    pub fn update_visibility(&mut self) {
        let update_fn = |data: &mut VisibleCellData, coord| {
            data.update(&self.world, coord);
        };
        let distance = if self.stats.day.is_empty() {
            Circle::new_squared(150)
        } else {
            Circle::new_squared(700)
        };
        self.visibility_grid.update_custom(
            Rgb24::new_grey(255),
            &self.world,
            distance,
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
        let mut local_rng = Isaac64Rng::seed_from_u64(self.seat_rng_seed);
        let boat_width1 = 3;
        let boat_width2 = 3;
        let boat_width3 = 2;
        let boat_length1 = 4;
        let boat_length2 = 6;
        let boat_length3 = 1;
        let boat_length4 = 4;
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
        for &coord in &boat_floor {
            self.world.spawn_boat_floor(coord + boat_coord);
        }
        self.world.spawn_boat_controls(boat_coord);
        self.world.components.boat.insert(boat_entity, boat);
        let _ = self
            .world
            .spatial_table
            .update_coord(boat_entity, boat_coord);
        if self.driving {
            let _ = self
                .world
                .spatial_table
                .update_coord(self.player_entity, boat_coord);
        }
        if !self.stats.day.is_empty() {
            let mut boat_floor = boat_floor.into_iter().collect::<Vec<_>>();
            boat_floor.sort();
            boat_floor.shuffle(&mut local_rng);
            for &npc in &self.passengers {
                if let Some(coord) = boat_floor.pop() {
                    let e = self.world.spawn_npc(coord + boat_coord, npc);
                    self.world.components.part_of_boat.insert(e, ());
                    let _ = self.world.spatial_table.update_layer(e, Layer::Feature);
                }
            }
        }
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

    pub fn cell_visibility_at_coord(&self, coord: Coord) -> CellVisibility<&VisibleCellData> {
        self.visibility_grid.get_visibility(coord)
    }

    fn rotate_boat(&mut self, rotate_direction: RotateDirection) -> Option<GameControlFlow> {
        let (boat_entity, boat) = self.world.components.boat.iter().next().unwrap();
        let step_radians = std::f64::consts::FRAC_PI_4;
        let delta_radians = match rotate_direction {
            RotateDirection::Left => -step_radians,
            RotateDirection::Right => step_radians,
        };
        let boat_next = boat.add_heading(Radians(delta_radians));
        let boat_coord = self.world.spatial_table.coord_of(boat_entity).unwrap();
        self.try_rasterize_boat(boat_entity, boat_next, boat_coord);
        self.pass_time();
        None
    }

    fn move_boat(&mut self, move_direction: MoveDirection) -> Option<GameControlFlow> {
        let (boat_entity, boat) = self.world.components.boat.iter().next().unwrap();
        let boat_coord = self.world.spatial_table.coord_of(boat_entity).unwrap();
        let (boat_next, delta) = match move_direction {
            MoveDirection::Forward => boat.step(),
            MoveDirection::Backward => boat.step_backwards(),
        };
        self.try_rasterize_boat(boat_entity, boat_next, boat_coord + delta);
        self.pass_time();
        self.spend_fuel();
        None
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

    fn player_walk(&mut self, direction: CardinalDirection) -> Option<GameControlFlow> {
        let player_coord = self.player_coord();
        let new_player_coord = player_coord + direction.coord();
        let layers = self
            .world
            .spatial_table
            .layers_at(new_player_coord)
            .cloned();
        if let Some(Layers {
            feature: Some(feature_entity),
            ..
        }) = layers
        {
            if let Some(state) = self.world.components.button.get_mut(feature_entity) {
                if !*state {
                    self.messages
                        .push(format!("You hear the sound of a distant gate opening"));
                }
                *state = true;
                self.world
                    .components
                    .tile
                    .insert(feature_entity, Tile::ButtonPressed);
                let world = &mut self.dungeon_state.as_mut().unwrap().world_tmp;
                let to_remove = world.components.gate.entities().collect::<Vec<_>>();
                for e in to_remove {
                    world.components.remove_entity(e);
                    world.spatial_table.remove(e);
                }
                return None;
            }
            // If the player bumps into a door, open the door
            if let Some(DoorState::Closed) = self.world.components.door_state.get(feature_entity) {
                self.open_door(feature_entity);
                return None;
            }
            // Don't let the player walk through solid entities
            if self.world.components.solid.contains(feature_entity) {
                if let Some(open_door_entity) =
                    self.open_door_entity_adjacent_to_coord(player_coord)
                {
                    self.close_door(open_door_entity);
                }
                return None;
            }
            {
                if let Some(&dungeon_index) = self.world.components.stairs_down.get(feature_entity)
                {
                    if let Err(UpdateError::OccupiedBy(entity)) = self
                        .world
                        .spatial_table
                        .update_coord(self.player_entity, new_player_coord)
                    {
                        if self.world.components.ghost.contains(entity) {
                            self.take_damage();
                            self.ghost_message();
                            if self.stats.health.is_empty() {
                                return Some(GameControlFlow::GameOver(
                                    GameOverReason::KilledByGhost,
                                ));
                            }
                        }
                        if self.world.components.beast.contains(entity) {
                            self.take_damage();
                            self.beast_message();
                            if self.stats.health.is_empty() {
                                return Some(GameControlFlow::GameOver(
                                    GameOverReason::KilledByBeast,
                                ));
                            }
                        }
                    }
                    self.enter_dungeon(dungeon_index);
                    return None;
                }
            }
            if self.world.components.stairs_up.contains(feature_entity) {
                self.exit_dungeon();
                return None;
            }
        }
        if let Some(Layers {
            water: Some(_),
            floor: None,
            feature: None,
            ..
        }) = layers
        {
            return None;
        }
        if let Some(Layers { boat: Some(_), .. }) = layers {
            self.has_been_on_boat = true;
        }
        if let Some(Layers {
            feature: Some(feature),
            ..
        }) = layers
        {
            if self.world.components.threshold.contains(feature) {
                self.has_crossed_threshold = true;
            }
            if let Some(victory) = self.world.components.grave.get(feature) {
                return Some(GameControlFlow::Menu(Menu {
                    choices: vec![MenuChoice::Leave],
                    text: victory.text(),
                    image: MenuImage::Grave,
                }));
            }
        }
        if let Err(UpdateError::OccupiedBy(entity)) = self
            .world
            .spatial_table
            .update_coord(self.player_entity, new_player_coord)
        {
            if self.world.components.ghost.contains(entity) {
                self.take_damage();
                self.ghost_message();
                if self.stats.health.is_empty() {
                    return Some(GameControlFlow::GameOver(GameOverReason::KilledByGhost));
                }
            }
            if self.world.components.beast.contains(entity) {
                self.take_damage();
                self.beast_message();
                if self.stats.health.is_empty() {
                    return Some(GameControlFlow::GameOver(GameOverReason::KilledByBeast));
                }
            }

            if self.world.components.unimportant_npc.contains(entity) {
                let text_options = vec![
                    "I think you would be happier if you went to the ocean.",
                    "Why don't you take a trip to the ocean. I hear it's wonderful this time of year.",
                    "It's time for you to head to the ocean.",
                ];
                let text_str = text_options.choose(&mut self.rng).unwrap();
                let text = format!("Townsperson:\n\n{}", text_str);
                return Some(GameControlFlow::Menu(Menu {
                    choices: vec![MenuChoice::SayNothing],
                    text,
                    image: MenuImage::Townsperson,
                }));
            }
            if !self.world.components.part_of_boat.contains(entity) {
                if let Some(&npc) = self.world.components.npc.get(entity) {
                    self.has_talked_to_npc = true;
                    let num_empty_seats = self.num_empty_seats();
                    let (text, choices) =
                        if num_empty_seats == 0 {
                            (
                        format!("I see that there is currently no room on your boat for me...\n\n"),
                        vec![MenuChoice::Leave],
                    )
                        } else {
                            let text = if num_empty_seats == 1 {
                                format!(
                                    "{}\n\n\nThere is currently 1 empty seat on your boat.\n\n",
                                    npc.text()
                                )
                            } else {
                                format!(
                                    "{}\n\n\nThere are currently {} empty seats on your boat.\n\n",
                                    npc.text(),
                                    num_empty_seats
                                )
                            };
                            (
                                text,
                                vec![
                                    MenuChoice::AddNpcToPassengers(entity),
                                    MenuChoice::DontAddNpcToPassengers,
                                ],
                            )
                        };
                    let image = MenuImage::Npc(npc);
                    return Some(GameControlFlow::Menu(Menu {
                        choices,
                        text,
                        image,
                    }));
                }
            }
            if let Some(&shop_i) = self.world.components.shop.get(entity) {
                let description = if shop_i == 0 {
                    "Welcome to the swamp. There is a city far to the east. You will need lots of fuel to make it \
                    there in your boat. Bring me junk from the nearby islands and I'll give you the fuel you need."
                } else {
                    "Long ago a gate was built to keep the water out of the city. As you can see it didn't work \
                    but while the gate is shut you won't be able to reach the ocean in your boat. Some of these old \
                    buildings have basements and I think the gate controls are down there somewhere."
                };
                let junk = self.stats.junk.current();
                let text =
                    format!("Innkeeper:\n\n{description}\n\n\nYou currently have {junk} junk.");
                let image = MenuImage::Shop;
                let choices = vec![
                    MenuChoice::BuyFuel {
                        cost: 2,
                        amount: 200,
                    },
                    MenuChoice::BuyCrewCapacity(self.num_seats),
                    MenuChoice::SleepUntilMorning(shop_i as u32),
                    MenuChoice::StayAtInnForever,
                    MenuChoice::Leave,
                ];
                return Some(GameControlFlow::Menu(Menu {
                    choices,
                    text,
                    image,
                }));
            }
        }
        if let Layers {
            item: Some(item), ..
        } = self.world.spatial_table.layers_at_checked(new_player_coord)
        {
            if !self.stats.junk.is_full() {
                self.stats.junk.increase(1);
                let entity_data = self.world.components.remove_entity_data(*item);
                self.world.spatial_table.remove(*item);
                if let Some(junk) = entity_data.junk {
                    let name = junk.name();
                    self.messages.push(format!("You pick up the {name}."));
                }
            }
        }
        self.pass_time();
        None
    }

    pub fn is_in_dungeon(&self) -> bool {
        self.dungeon_state.is_some()
    }

    fn exit_dungeon(&mut self) {
        use std::mem;
        let player_data = self.world.components.remove_entity_data(self.player_entity);
        self.world.spatial_table.remove(self.player_entity);
        self.world.entity_allocator.free(self.player_entity);
        let dungeon_state = self.dungeon_state.take().unwrap();
        let dungeon_world = mem::replace(&mut self.world, dungeon_state.world_tmp);
        self.player_entity = self.world.insert_entity_data(
            Location {
                coord: dungeon_state.return_coord,
                layer: Some(Layer::Character),
            },
            player_data,
        );
        self.dungeons[dungeon_state.dungeon_index] = Some(Dungeon {
            world: dungeon_world,
            spawn: dungeon_state.dungeon_spawn,
        });
        self.visibility_grid = dungeon_state.visibility_grid;
    }

    fn enter_dungeon(&mut self, dungeon_index: usize) {
        use std::mem;
        let return_coord = self.player_coord();
        let player_data = self.world.components.remove_entity_data(self.player_entity);
        self.world.spatial_table.remove(self.player_entity);
        self.world.entity_allocator.free(self.player_entity);
        let mut dungeon = self.dungeons[dungeon_index].take().unwrap();
        self.player_entity = dungeon.world.insert_entity_data(
            Location {
                coord: dungeon.spawn,
                layer: Some(Layer::Character),
            },
            player_data,
        );
        let world_tmp = mem::replace(&mut self.world, dungeon.world);
        let visibility_grid = mem::replace(
            &mut self.visibility_grid,
            VisibilityGrid::new(self.world.spatial_table.grid_size()),
        );
        let dungeon_state = DungeonState {
            world_tmp,
            dungeon_index,
            return_coord,
            dungeon_spawn: dungeon.spawn,
            visibility_grid,
        };
        self.dungeon_state = Some(dungeon_state);
    }

    fn is_player_on_driving_coord(&self) -> bool {
        let player_coord = self.player_coord();
        let (boat_entity, _boat) = self.world.components.boat.iter().next().unwrap();
        let boat_coord = self.world.spatial_table.coord_of(boat_entity).unwrap();
        player_coord == boat_coord
    }

    fn check_control_flow(&self) -> Option<GameControlFlow> {
        if self.stats.fuel.is_empty() {
            return Some(GameControlFlow::GameOver(GameOverReason::OutOfFuel));
        }
        None
    }

    fn npc_turn(&mut self) -> Option<GameControlFlow> {
        if self.is_player_outside_at_night() {
            let ghost_entities = self.world.components.ghost.entities().collect::<Vec<_>>();
            let player_coord = self.player_coord();
            for entity in ghost_entities {
                let coord = self.world.spatial_table.coord_of(entity).unwrap();
                if let Some(dest) = line_2d::coords_between(coord, player_coord).skip(1).next() {
                    if dest == player_coord {
                        self.take_damage();
                        self.ghost_message();
                        if self.stats.health.is_empty() {
                            return Some(GameControlFlow::GameOver(GameOverReason::KilledByGhost));
                        }
                    } else {
                        let _ = self.world.spatial_table.update_coord(entity, dest);
                    }
                }
            }
        }
        {
            struct C<'a> {
                components: &'a Components,
                spatial_table: &'a SpatialTable,
            }
            impl<'a> distance_map::CanEnter for C<'a> {
                fn can_enter(&self, coord: Coord) -> bool {
                    if let Some(&Layers {
                        feature: Some(feature),
                        ..
                    }) = self.spatial_table.layers_at(coord)
                    {
                        if self.components.solid.contains(feature) {
                            return false;
                        }
                    }
                    true
                }
            }
            let c = C {
                components: &self.world.components,
                spatial_table: &self.world.spatial_table,
            };
            self.ai_ctx.distance_map.clear();
            self.ai_ctx.distance_map.add(self.player_coord());
            self.ai_ctx
                .distance_map
                .populate_approach(&c, 20, &mut self.world.distance_map);
            let beasts = self.world.components.beast.entities().collect::<Vec<_>>();
            for entity in beasts {
                if let Some(coord) = self.world.spatial_table.coord_of(entity) {
                    if let Some(direction) =
                        self.world.distance_map.direction_to_best_neighbour(coord)
                    {
                        let destination = coord + direction.coord();
                        if destination == self.player_coord() {
                            self.take_damage();
                            self.beast_message();
                            if self.stats.health.is_empty() {
                                return Some(GameControlFlow::GameOver(
                                    GameOverReason::KilledByBeast,
                                ));
                            }
                        } else {
                            let _ = self.world.spatial_table.update_coord(entity, destination);
                        }
                    }
                }
            }
        }
        None
    }

    fn ghost_message(&mut self) {
        self.messages.push(format!(
            "A chill runs down your spine. The ghost deals you 1 damage."
        ));
    }

    fn beast_message(&mut self) {
        self.messages.push(format!("The beast deals you 1 damage."));
    }

    fn add_npc_to_passengers(&mut self, entity: Entity) {
        let entity_data = self.world.components.remove_entity_data(entity);
        self.world.spatial_table.remove(entity);
        let npc = entity_data.npc.unwrap();
        self.passengers.push(npc);
    }

    #[must_use]
    pub(crate) fn handle_tick(
        &mut self,
        _since_last_tick: Duration,
        _config: &Config,
    ) -> Option<GameControlFlow> {
        if let Layers {
            water: Some(water), ..
        } = self
            .world
            .spatial_table
            .layers_at_checked(self.player_coord())
        {
            if self.world.components.ocean.contains(*water) {
                return Some(GameControlFlow::Win);
            }
        }
        None
    }

    fn handle_ability(&mut self, index: u8) -> Option<GameControlFlow> {
        let passenger_index = index as usize - 1;
        if let Some(&npc) = self.passengers.get(passenger_index) {
            let aim_needed = match npc {
                Npc::Physicist | Npc::Soldier => true,
            };
            if aim_needed {
                return Some(GameControlFlow::Aim(npc));
            }
        }
        None
    }

    #[must_use]
    pub(crate) fn handle_input(
        &mut self,
        input: Input,
        _config: &Config,
    ) -> Result<Option<GameControlFlow>, ActionError> {
        let game_control_flow = if self.driving {
            match input {
                Input::Walk(CardinalDirection::East) => self.rotate_boat(RotateDirection::Right),
                Input::Walk(CardinalDirection::West) => self.rotate_boat(RotateDirection::Left),
                Input::Walk(CardinalDirection::North) => self.move_boat(MoveDirection::Forward),
                Input::Walk(CardinalDirection::South) => self.move_boat(MoveDirection::Backward),
                Input::DriveToggle => {
                    self.driving = false;
                    None
                }
                Input::Wait => {
                    self.pass_time();
                    None
                }
                Input::Ability(i) => self.handle_ability(i),
            }
        } else {
            match input {
                Input::Walk(direction) => self.player_walk(direction),
                Input::DriveToggle => {
                    if self.is_player_on_driving_coord() {
                        self.driving = true;
                    }
                    None
                }
                Input::Wait => {
                    self.pass_time();
                    None
                }
                Input::Ability(i) => self.handle_ability(i),
            }
        };
        if game_control_flow.is_some() {
            return Ok(game_control_flow);
        }
        let game_control_flow = self.npc_turn();
        if game_control_flow.is_some() {
            return Ok(game_control_flow);
        }
        self.update_visibility();
        Ok(self.check_control_flow())
    }

    fn stay_at_inn_forever(&self) -> GameControlFlow {
        let text = "Innkeeper:\n\n\
            So you find yourself wishing to remain at the inn and abandon your quest \
            to reach the ocean?\n\nAre you sure?"
            .to_string();
        let choices = vec![MenuChoice::AbandonQuest, MenuChoice::ChangeMind];
        let image = MenuImage::Shop;
        GameControlFlow::Menu(Menu {
            text,
            choices,
            image,
        })
    }

    fn cant_afford(&self) -> GameControlFlow {
        let text = "Innkeeper:\n\n\
            You can't afford that."
            .to_string();
        let choices = vec![MenuChoice::Okay];
        let image = MenuImage::Shop;
        GameControlFlow::Menu(Menu {
            text,
            choices,
            image,
        })
    }

    fn buy_fuel(&mut self, amount: u32, cost: u32) -> Option<GameControlFlow> {
        if self.stats.junk.current() < cost {
            return Some(self.cant_afford());
        }
        self.stats.junk.decrease(cost);
        self.stats.fuel.increase(amount);
        self.messages
            .push(format!("You trade {cost} junk for {amount} fuel."));
        None
    }

    fn buy_passenger_capacity(&mut self, cost: u32) -> Option<GameControlFlow> {
        if self.stats.junk.current() < cost {
            return Some(self.cant_afford());
        }
        self.stats.junk.decrease(cost);
        self.num_seats += 1;
        self.messages.push(format!(
            "You trade {cost} junk for an additional passenger space."
        ));
        None
    }

    fn is_coord_visible(&self, coord: Coord) -> bool {
        match self.cell_visibility_at_coord(coord) {
            CellVisibility::Current { .. } => true,
            CellVisibility::Previous(_) => true,
            _ => false,
        }
    }

    fn ability_soldier(&mut self, target: Coord) -> Result<Option<GameControlFlow>, String> {
        let radius = 4;
        let blob = procgen::blob(target, Size::new(radius, radius), &mut self.rng);
        let player_coord = self.player_coord();
        let mut to_remove = Vec::new();
        for coord in blob.inside {
            if coord == player_coord {
                return Err(format!("Within minimum safe distance. Refusing to fire"));
            }
            if let Some(&layers) = self.world.spatial_table.layers_at(coord) {
                if let Some(floor) = layers.floor {
                    self.world.components.tile.insert(floor, Tile::BurntFloor);
                }
                if let Some(character) = layers.character {
                    if self.world.components.destructible.contains(character) {
                        to_remove.push(character);
                    }
                }
                if let Some(feature) = layers.character {
                    if self.world.components.destructible.contains(feature) {
                        to_remove.push(feature);
                    }
                }
            }
        }
        for e in to_remove {
            self.world.components.remove_entity(e);
            self.world.spatial_table.remove(e);
        }
        Ok(None)
    }
    fn ability_physicist(&mut self, target: Coord) -> Result<Option<GameControlFlow>, String> {
        if self.driving {
        } else {
            let mut can_teleport = true;
            if let Some(&layers) = self.world.spatial_table.layers_at(target) {
                if !(layers.water.is_none() && layers.character.is_none()) {
                    can_teleport = false;
                }
                if let Some(feature) = layers.feature {
                    if self.world.components.solid.contains(feature) {
                        can_teleport = false;
                    }
                }
            } else {
                can_teleport = false;
            }
            if can_teleport {
                let _ = self
                    .world
                    .spatial_table
                    .update_coord(self.player_entity, target);
            } else {
                return Err(format!("Destination is not empty!"));
            }
        }
        Ok(None)
    }

    pub(crate) fn handle_aim(&mut self, npc: Npc, coord: Coord) -> Option<GameControlFlow> {
        self.update_visibility();
        if self.is_coord_visible(coord) {
            let result = match npc {
                Npc::Soldier => self.ability_soldier(coord),
                Npc::Physicist => self.ability_physicist(coord),
            };
            match result {
                Ok(maybe_cf) => {
                    self.pass_time();
                    self.npc_turn();
                    self.update_visibility();
                    maybe_cf
                }
                Err(message) => {
                    self.messages.push(message);
                    None
                }
            }
        } else {
            self.messages.push(format!("Can't see the target."));
            None
        }
    }

    pub(crate) fn handle_choice(&mut self, choice: MenuChoice) -> Option<GameControlFlow> {
        match choice {
            MenuChoice::DontAddNpcToPassengers
            | MenuChoice::Leave
            | MenuChoice::SayNothing
            | MenuChoice::ChangeMind => (),
            MenuChoice::AddNpcToPassengers(entity) => self.add_npc_to_passengers(entity),
            MenuChoice::SleepUntilMorning(_) => self.start_day(),
            MenuChoice::AbandonQuest => {
                return Some(GameControlFlow::GameOver(GameOverReason::Abandoned));
            }
            MenuChoice::StayAtInnForever => return Some(self.stay_at_inn_forever()),
            MenuChoice::BuyFuel { cost, amount } => {
                if let Some(cf) = self.buy_fuel(amount, cost) {
                    return Some(cf);
                }
            }
            MenuChoice::BuyCrewCapacity(cost) => {
                if let Some(cf) = self.buy_passenger_capacity(cost) {
                    return Some(cf);
                }
            }
            _ => (),
        }
        // TODO this should be a function
        let (boat_entity, boat) = self.world.components.boat.iter().next().unwrap();
        let boat_coord = self.world.spatial_table.coord_of(boat_entity).unwrap();
        if !self.try_rasterize_boat(boat_entity, boat.clone(), boat_coord) {
            panic!("failed to create the boat");
        }
        self.update_visibility();
        None
    }
}
