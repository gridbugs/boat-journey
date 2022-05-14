use crate::behaviour::Agent;
use crate::visibility::Light;
use crate::{
    world::EntityData,
    world::{Layer, Location, MeleeWeapon, RangedWeapon},
    Tile, World,
};
use gridbugs::{
    coord_2d::{Coord, Size},
    direction::{Axis, Directions},
    entity_table::{ComponentTable, Entity},
    rgb_int::Rgb24,
    shadowcast::vision_distance::Circle,
};
use rand::{seq::SliceRandom, Rng};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

const AREA_SIZE: Size = Size::new_u16(27, 20);

pub struct Terrain {
    pub world: World,
    pub player: Entity,
    pub agents: ComponentTable<Agent>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TerrainState {
    ranged_weapons: Vec<RangedWeapon>,
    chainsaw_floors: HashSet<u32>,
}

impl TerrainState {
    pub fn new<R: Rng>(rng: &mut R) -> Self {
        let mut ranged_weapons = vec![
            RangedWeapon::Shotgun,
            RangedWeapon::Rifle,
            RangedWeapon::Railgun,
            RangedWeapon::GausCannon,
            RangedWeapon::LifeStealer,
            RangedWeapon::Oxidiser,
            RangedWeapon::Shotgun,
            RangedWeapon::Rifle,
            RangedWeapon::Railgun,
            RangedWeapon::GausCannon,
            RangedWeapon::LifeStealer,
            RangedWeapon::Oxidiser,
        ];
        ranged_weapons.shuffle(rng);
        let mut floors = (1..=5).collect::<Vec<_>>();
        floors.shuffle(rng);
        let mut chainsaw_floors = HashSet::new();
        for _ in 0..2 {
            chainsaw_floors.insert(floors.pop().unwrap());
        }
        Self {
            ranged_weapons,
            chainsaw_floors,
        }
    }
}

#[allow(dead_code)]
pub fn from_str(s: &str, player_data: EntityData) -> Terrain {
    let rows = s.split('\n').filter(|s| !s.is_empty()).collect::<Vec<_>>();
    let size = Size::new_u16(rows[0].len() as u16, rows.len() as u16);
    let mut world = World::new(AREA_SIZE, 0);
    let mut agents = ComponentTable::default();
    let mut player_data = Some(player_data);
    let mut player = None;
    for (y, row) in rows.iter().enumerate() {
        for (x, ch) in row.chars().enumerate() {
            if ch.is_control() {
                continue;
            }
            let coord = Coord::new(x as i32, y as i32);
            match ch {
                '.' => {
                    world.spawn_floor(coord);
                }
                '#' => {
                    world.spawn_floor(coord);
                    world.spawn_wall(coord);
                }
                '+' => {
                    world.spawn_floor(coord);
                    world.spawn_wall(coord);
                }
                '%' => {
                    world.spawn_floor(coord);
                    world.spawn_window(coord, Axis::X);
                }
                '>' => {
                    world.spawn_stairs(coord);
                    world.spawn_floor(coord);
                }
                'z' => {
                    let entity = world.spawn_zombie(coord);
                    agents.insert(entity, Agent::new(size));
                    world.spawn_floor(coord);
                }
                't' => {
                    let entity = world.spawn_tank(coord);
                    agents.insert(entity, Agent::new(size));
                    world.spawn_floor(coord);
                }
                's' => {
                    let entity = world.spawn_skeleton(coord);
                    agents.insert(entity, Agent::new(size));
                    world.spawn_floor(coord);
                }
                'b' => {
                    let entity = world.spawn_boomer(coord);
                    agents.insert(entity, Agent::new(size));
                    world.spawn_floor(coord);
                }
                'u' => {
                    world.spawn_upgrade(coord);
                    world.spawn_floor(coord);
                }
                'm' => {
                    world.spawn_map(coord);
                    world.spawn_floor(coord);
                }
                '$' => {
                    world.spawn_credit(coord, 2);
                    world.spawn_floor(coord);
                }
                'h' => {
                    world.spawn_medkit(coord);
                    world.spawn_floor(coord);
                }
                '0'..='5' => {
                    use RangedWeapon::*;
                    let weapon = match ch {
                        '0' => Shotgun,
                        '1' => Railgun,
                        '2' => Rifle,
                        '3' => GausCannon,
                        '4' => Oxidiser,
                        '5' => LifeStealer,
                        _ => panic!(),
                    };
                    world.spawn_ranged_weapon(coord, weapon);
                    world.spawn_floor(coord);
                }
                '6'..='6' => {
                    use MeleeWeapon::*;
                    let weapon = match ch {
                        '6' => Chainsaw,
                        _ => panic!(),
                    };
                    world.spawn_melee_weapon(coord, weapon);
                    world.spawn_floor(coord);
                }

                '@' => {
                    world.spawn_floor(coord);
                    let location = Location {
                        coord,
                        layer: Some(Layer::Character),
                    };
                    player = Some(world.insert_entity_data(location, player_data.take().unwrap()));
                }
                ' ' => (),
                _ => log::warn!(
                    "unexpected char in terrain: {} ({})",
                    ch.escape_unicode(),
                    ch
                ),
            }
        }
    }
    let player = player.expect("didn't create player");
    Terrain {
        world,
        player,
        agents,
    }
}

pub struct SpaceStationSpec {
    pub demo: bool,
}

pub fn space_station_first_floor<R: Rng>(
    player_data: EntityData,
    spec: &SpaceStationSpec,
    rng: &mut R,
) -> Terrain {
    const SHIP_SIZE: Size = Size::new_u16(10, 10);
    const SHIP_OFFSET: Coord = Coord { x: 5, y: 5 };
    let grid = procgen::generate(
        procgen::Spec {
            size: SHIP_SIZE,
            small: true,
        },
        rng,
    );
    let mut world = World::new(AREA_SIZE, 0);
    let agents = ComponentTable::default();
    let mut player_data = Some(player_data);
    let mut player = None;
    let mut door_coord = None;
    let mut above_door = None;
    for (coord, cell) in grid.enumerate() {
        let coord = coord + SHIP_OFFSET;
        use procgen::GameCell;
        match cell {
            GameCell::Wall => {
                world.spawn_floor(coord);
                world.spawn_wall(coord);
            }
            GameCell::Floor => {
                world.spawn_floor(coord);
            }
            GameCell::Space => {}
            GameCell::Door(axis) => {
                above_door = Some(coord - Coord::new(0, 1));
                door_coord = Some(coord);
                world.spawn_floor(coord);
                world.spawn_door(coord, *axis);
            }
            GameCell::Window(axis) => {
                world.spawn_floor(coord);
                world.spawn_window(coord, *axis);
            }
            GameCell::Stairs => {
                if !spec.demo {
                    world.spawn_stairs(coord);
                }
                world.spawn_floor(coord);
            }
            GameCell::Spawn => {
                world.spawn_floor(coord);
                if spec.demo {
                    let location = Location {
                        coord: Coord::new(10000, 10000),
                        layer: None,
                    };
                    player = Some(world.insert_entity_data(location, player_data.take().unwrap()));
                } else {
                    let location = Location {
                        coord,
                        layer: Some(Layer::Character),
                    };
                    player = Some(world.insert_entity_data(location, player_data.take().unwrap()));
                }
            }
        }
    }
    let above_door = above_door.unwrap();
    let starter_gun = if rng.gen::<bool>() {
        RangedWeapon::Shotgun
    } else {
        RangedWeapon::Rifle
    };
    world.spawn_ranged_weapon(above_door, starter_gun);
    let door_coord = door_coord.unwrap();
    world.components.tile.insert(
        world
            .spatial_table
            .layers_at_checked(door_coord - Coord::new(4, 0))
            .feature
            .unwrap(),
        Tile::WallText0,
    );
    world.components.tile.insert(
        world
            .spatial_table
            .layers_at_checked(door_coord - Coord::new(3, 0))
            .feature
            .unwrap(),
        Tile::WallText1,
    );
    world.components.tile.insert(
        world
            .spatial_table
            .layers_at_checked(door_coord - Coord::new(2, 0))
            .feature
            .unwrap(),
        Tile::WallText2,
    );
    world.components.tile.insert(
        world
            .spatial_table
            .layers_at_checked(door_coord - Coord::new(1, 0))
            .feature
            .unwrap(),
        Tile::WallText3,
    );

    let player = player.expect("didn't create player");
    Terrain {
        world,
        player,
        agents,
    }
}

pub fn space_station<R: Rng>(
    level: u32,
    player_data: EntityData,
    spec: &SpaceStationSpec,
    terrain_state: &mut TerrainState,
    rng: &mut R,
) -> Terrain {
    if !spec.demo {
        if level == 0 {
            return space_station_first_floor(player_data, spec, rng);
        }
        if level == FINAL_LEVEL {
            return space_station_last_level(FINAL_LEVEL, player_data, spec, terrain_state, rng);
        }
    }
    const SHIP_SIZE: Size = Size::new_u16(20, 14);
    const SHIP_OFFSET: Coord = Coord { x: 1, y: 1 };
    let grid = procgen::generate(
        procgen::Spec {
            size: SHIP_SIZE,
            small: false,
        },
        rng,
    );
    let mut world = World::new(AREA_SIZE, level);
    let mut agents = ComponentTable::default();
    let mut player_data = Some(player_data);
    let mut player = None;
    let mut empty_coords = Vec::new();
    for (coord, cell) in grid.enumerate() {
        let coord = coord + SHIP_OFFSET;
        use procgen::GameCell;
        match cell {
            GameCell::Wall => {
                world.spawn_floor(coord);
                world.spawn_wall(coord);
            }
            GameCell::Floor => {
                empty_coords.push(coord);
                world.spawn_floor(coord);
            }
            GameCell::Space => {}
            GameCell::Door(axis) => {
                world.spawn_floor(coord);
                world.spawn_door(coord, *axis);
            }
            GameCell::Window(axis) => {
                world.spawn_floor(coord);
                world.spawn_window(coord, *axis);
            }
            GameCell::Stairs => {
                if !spec.demo {
                    world.spawn_stairs(coord);
                }
                world.spawn_floor(coord);
            }
            GameCell::Spawn => {
                world.spawn_floor(coord);
                if spec.demo {
                    let location = Location {
                        coord: Coord::new(10000, 10000),
                        layer: None,
                    };
                    player = Some(world.insert_entity_data(location, player_data.take().unwrap()));
                } else {
                    let location = Location {
                        coord,
                        layer: Some(Layer::Character),
                    };
                    player = Some(world.insert_entity_data(location, player_data.take().unwrap()));
                }
            }
        }
    }
    empty_coords.shuffle(rng);
    if spec.demo {
        for _ in 0..2 {
            if let Some(coord) = empty_coords.pop() {
                let entity = world.spawn_zombie(coord);
                agents.insert(entity, Agent::new(AREA_SIZE));
            }
        }
    } else {
        spawn_items(level, &mut empty_coords, &mut world, terrain_state, rng);
    }
    let player = player.expect("didn't create player");
    Terrain {
        world,
        player,
        agents,
    }
}

fn space_station_last_level<R: Rng>(
    level: u32,
    player_data: EntityData,
    spec: &SpaceStationSpec,
    terrain_state: &mut TerrainState,
    rng: &mut R,
) -> Terrain {
    const SHIP_SIZE: Size = Size::new_u16(20, 14);
    const SHIP_OFFSET: Coord = Coord { x: 1, y: 1 };
    let grid = procgen::generate(
        procgen::Spec {
            size: SHIP_SIZE,
            small: false,
        },
        rng,
    );
    let mut world = World::new(AREA_SIZE, FINAL_LEVEL);
    let agents = ComponentTable::default();
    let mut player_data = Some(player_data);
    let mut player = None;
    let mut empty_coords = Vec::new();
    let mut stairs_coord = None;
    for (coord, cell) in grid.enumerate() {
        let coord = coord + SHIP_OFFSET;
        use procgen::GameCell;
        match cell {
            GameCell::Wall => {
                world.spawn_floor(coord);
                world.spawn_wall(coord);
            }
            GameCell::Floor => {
                empty_coords.push(coord);
                world.spawn_floor(coord);
            }
            GameCell::Space => {}
            GameCell::Door(axis) => {
                world.spawn_floor(coord);
                world.spawn_door(coord, *axis);
            }
            GameCell::Window(axis) => {
                world.spawn_floor(coord);
                world.spawn_window(coord, *axis);
            }
            GameCell::Stairs => {
                world.spawn_floor(coord);
                world.spawn_stairs(coord + Coord::new(1, 0));
                stairs_coord = Some(coord + Coord::new(1, 0));
            }
            GameCell::Spawn => {
                world.spawn_floor(coord);
                if spec.demo {
                    let location = Location {
                        coord: Coord::new(10000, 10000),
                        layer: None,
                    };
                    player = Some(world.insert_entity_data(location, player_data.take().unwrap()));
                } else {
                    let location = Location {
                        coord,
                        layer: Some(Layer::Character),
                    };
                    player = Some(world.insert_entity_data(location, player_data.take().unwrap()));
                }
            }
        }
    }
    let stairs_coord = stairs_coord.unwrap();
    empty_coords.shuffle(rng);
    let mut empty_coords = empty_coords
        .into_iter()
        .filter(|&c| c != stairs_coord)
        .collect::<Vec<_>>();
    world.components.tile.insert(
        world
            .spatial_table
            .layers_at_checked(stairs_coord - Coord::new(2, 0))
            .floor
            .unwrap(),
        Tile::FuelText0,
    );
    world.components.tile.insert(
        world
            .spatial_table
            .layers_at_checked(stairs_coord - Coord::new(1, 0))
            .floor
            .unwrap(),
        Tile::FuelText1,
    );
    world.components.tile.insert(
        world
            .spatial_table
            .layers_at_checked(stairs_coord)
            .feature
            .unwrap(),
        Tile::FuelHatch,
    );
    let fuel_light = Light {
        colour: Rgb24::new(0, 0, 255),
        vision_distance: Circle::new_squared(120),
        diminish_numerator: 1,
        diminish_denominator: 8,
    };
    world.components.light.insert(
        world
            .spatial_table
            .layers_at_checked(stairs_coord)
            .feature
            .unwrap(),
        fuel_light,
    );
    spawn_items(level, &mut empty_coords, &mut world, terrain_state, rng);
    let player = player.expect("didn't create player");
    Terrain {
        world,
        player,
        agents,
    }
}

struct EnemyCounts {
    zombie: Vec<usize>,
    skeleton: Vec<usize>,
    boomer: Vec<usize>,
    tank: Vec<usize>,
}

impl EnemyCounts {
    fn new() -> Self {
        Self {
            zombie: vec![5, 7, 7, 8, 8],
            skeleton: vec![1, 1, 1, 2, 2],
            boomer: vec![0, 0, 1, 1, 2],
            tank: vec![0, 0, 0, 1, 2],
        }
    }
}

fn spawn_items<R: Rng>(
    level: u32,
    empty_coords: &mut Vec<Coord>,
    world: &mut World,
    terrain_state: &mut TerrainState,
    _rng: &mut R,
) {
    for _ in 0..2 {
        if let Some(coord) = empty_coords.pop() {
            world.spawn_credit(coord, 2);
        }
    }
    for _ in 0..4 {
        if let Some(coord) = empty_coords.pop() {
            world.spawn_credit(coord, 1);
        }
    }
    for _ in 0..1 {
        if let Some(coord) = empty_coords.pop() {
            world.spawn_medkit(coord);
        }
    }
    for _ in 0..2 {
        if let Some(coord) = empty_coords.pop() {
            world.spawn_ranged_weapon(coord, terrain_state.ranged_weapons.pop().unwrap());
        }
    }
    if terrain_state.chainsaw_floors.contains(&level) {
        if let Some(coord) = empty_coords.pop() {
            world.spawn_melee_weapon(coord, MeleeWeapon::Chainsaw);
        }
    }
    'outer1: for (i, &coord) in empty_coords.iter().enumerate() {
        for direction in Directions {
            let nei = coord + direction.coord();
            if let Some(layers) = world.spatial_table.layers_at(nei) {
                if layers.feature.is_some() {
                    continue 'outer1;
                }
            }
        }
        world.spawn_upgrade(coord);
        empty_coords.swap_remove(i);
        break;
    }
    'outer2: for (i, &coord) in empty_coords.iter().enumerate() {
        for direction in Directions {
            let nei = coord + direction.coord();
            if let Some(layers) = world.spatial_table.layers_at(nei) {
                if layers.feature.is_some() {
                    continue 'outer2;
                }
            }
        }
        world.spawn_map(coord);
        empty_coords.swap_remove(i);
        break;
    }
    let index = level as usize - 1;
    let enemy_count = EnemyCounts::new();
    for _ in 0..enemy_count.zombie[index] {
        if let Some(coord) = empty_coords.pop() {
            world.spawn_zombie(coord);
        }
    }
    for _ in 0..enemy_count.skeleton[index] {
        if let Some(coord) = empty_coords.pop() {
            world.spawn_skeleton(coord);
        }
    }
    for _ in 0..enemy_count.boomer[index] {
        if let Some(coord) = empty_coords.pop() {
            world.spawn_boomer(coord);
        }
    }
    for _ in 0..enemy_count.tank[index] {
        if let Some(coord) = empty_coords.pop() {
            world.spawn_tank(coord);
        }
    }
}

pub const FINAL_LEVEL: u32 = 5;
