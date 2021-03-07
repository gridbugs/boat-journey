use crate::behaviour::Agent;
use crate::visibility::Light;
use crate::{
    world::EntityData,
    world::{Layer, Location},
    Tile, World,
};
use entity_table::{ComponentTable, Entity};
use grid_2d::CoordIter;
use grid_2d::{coord_2d::Axis, Coord, Size};
use rand::{
    seq::{IteratorRandom, SliceRandom},
    Rng, SeedableRng,
};
use rand_isaac::Isaac64Rng;
use rational::Rational;
use rgb24::Rgb24;
use shadowcast::vision_distance::Circle;

pub struct Terrain {
    pub world: World,
    pub player: Entity,
    pub agents: ComponentTable<Agent>,
}

#[allow(dead_code)]
pub fn from_str<R: Rng>(s: &str, player_data: EntityData, rng: &mut R) -> Terrain {
    let rows = s.split('\n').filter(|s| !s.is_empty()).collect::<Vec<_>>();
    let size = Size::new_u16(rows[0].len() as u16, rows.len() as u16);
    let mut world = World::new(size, 0);
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
                    world.spawn_door(coord, Axis::X);
                }
                '>' => {
                    world.spawn_stairs(coord);
                }
                'z' => {
                    let entity = world.spawn_zombie(coord, rng);
                    agents.insert(entity, Agent::new(size));
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

#[derive(Clone, Copy)]
enum NpcType {}

fn spawn_npc<R: Rng>(world: &mut World, npc_type: NpcType, coord: Coord, rng: &mut R) -> Entity {
    match npc_type {}
}

const ENEMY_TYPES: &[NpcType] = &[];

#[derive(Clone, Copy)]
enum Item {
    CreditChip,
}

impl Item {
    fn spawn(self, world: &mut World, coord: Coord, special: bool) {
        todo!()
    }
}

const ALL_ITEMS: &[Item] = &[Item::CreditChip];
const BALANCED_ITEMS: &[Item] = &[Item::CreditChip];

pub struct SpaceStationSpec {
    pub demo: bool,
}

pub fn space_station_first_floor<R: Rng>(
    player_data: EntityData,
    spec: &SpaceStationSpec,
    rng: &mut R,
) -> Terrain {
    const AREA_SIZE: Size = Size::new_u16(27, 20);
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
    rng: &mut R,
) -> Terrain {
    if !spec.demo {
        if level == 0 {
            return space_station_first_floor(player_data, spec, rng);
        }
        if level == FINAL_LEVEL {
            return space_station_last_level(player_data, spec, rng);
        }
    }
    const AREA_SIZE: Size = Size::new_u16(27, 20);
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
                if spec.demo {
                    world.spawn_floor(coord);
                } else {
                    world.spawn_stairs(coord);
                }
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
    let num_npcs = if spec.demo { 2 } else { level * 3 + 3 };
    for _ in 0..num_npcs {
        if let Some(coord) = empty_coords.pop() {
            let entity = world.spawn_zombie(coord, rng);
            agents.insert(entity, Agent::new(AREA_SIZE));
        }
    }
    let player = player.expect("didn't create player");
    Terrain {
        world,
        player,
        agents,
    }
}

fn space_station_last_level<R: Rng>(
    player_data: EntityData,
    spec: &SpaceStationSpec,
    rng: &mut R,
) -> Terrain {
    const AREA_SIZE: Size = Size::new_u16(27, 20);
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
    let mut agents = ComponentTable::default();
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
    let num_npcs = if spec.demo { 2 } else { FINAL_LEVEL * 3 + 3 };
    for _ in 0..num_npcs {
        if let Some(coord) = empty_coords.pop() {
            let entity = world.spawn_zombie(coord, rng);
            agents.insert(entity, Agent::new(AREA_SIZE));
        }
    }
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
        diminish: Rational {
            numerator: 1,
            denominator: 8,
        },
    };
    world.components.light.insert(
        world
            .spatial_table
            .layers_at_checked(stairs_coord)
            .feature
            .unwrap(),
        fuel_light,
    );
    let player = player.expect("didn't create player");
    Terrain {
        world,
        player,
        agents,
    }
}

pub const FINAL_LEVEL: u32 = 5;
