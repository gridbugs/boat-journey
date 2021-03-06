use crate::behaviour::Agent;
use crate::{
    world::EntityData,
    world::{Layer, Location},
    World,
};
use entity_table::{ComponentTable, Entity};
use grid_2d::CoordIter;
use grid_2d::{coord_2d::Axis, Coord, Size};
use rand::{
    seq::{IteratorRandom, SliceRandom},
    Rng, SeedableRng,
};
use rand_isaac::Isaac64Rng;

use rgb24::Rgb24;

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
                '*' => {
                    world.spawn_floor(coord);
                    world.spawn_light(coord, Rgb24::new(187, 187, 187));
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

pub fn space_station<R: Rng>(
    star_rng_seed: u64,
    level: u32,
    player_data: EntityData,
    rng: &mut R,
) -> Terrain {
    const AREA_SIZE: Size = Size::new_u16(27, 20);
    const SHIP_SIZE: Size = Size::new_u16(20, 14);
    const SHIP_OFFSET: Coord = Coord { x: 1, y: 1 };
    let grid = procgen::generate(procgen::Spec { size: SHIP_SIZE }, rng);
    let mut world = World::new(AREA_SIZE, 0);
    let mut agents = ComponentTable::default();
    let mut player_data = Some(player_data);
    let mut player = None;
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
                world.spawn_floor(coord);
                world.spawn_door(coord, *axis);
            }
            GameCell::Window(axis) => {
                world.spawn_floor(coord);
                world.spawn_window(coord, *axis);
            }
            GameCell::Stairs => {
                world.spawn_stairs(coord);
            }
            GameCell::Spawn => {
                world.spawn_floor(coord);
                let location = Location {
                    coord,
                    layer: Some(Layer::Character),
                };
                player = Some(world.insert_entity_data(location, player_data.take().unwrap()));
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

pub const FINAL_LEVEL: u32 = 5;
