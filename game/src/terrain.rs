use crate::{
    world::{
        data::{Boat, EntityData},
        spatial::{Layer, Location},
        World,
    },
    Entity,
};
use gridbugs::{
    coord_2d::{Coord, Size},
    entity_table::entity_data,
};
use rand::Rng;
use vector::Radians;

pub struct Terrain {
    pub world: World,
    pub player_entity: Entity,
}

impl Terrain {
    fn text_island(offset: Coord, world: &mut World) {
        let txt = include_str!("island.txt");
        for (y, row) in txt.split('\n').enumerate() {
            for (x, ch) in row.chars().enumerate() {
                let coord = offset + Coord::new(x as i32, y as i32);
                match ch {
                    ' ' => (),
                    '%' => {
                        world.spawn_rock(coord);
                    }
                    '.' => {
                        world.spawn_floor(coord);
                    }
                    '#' => {
                        world.spawn_wall(coord);
                    }
                    '+' => {
                        world.spawn_door(coord);
                    }
                    _ => panic!("unexpected char: {}", ch),
                }
            }
        }
    }

    pub fn generate<R: Rng>(player_data: EntityData, rng: &mut R) -> Self {
        let mut world = World::new(Size::new(1000, 500));
        let player_entity = world.insert_entity_data(
            Location {
                coord: Coord::new(50, 50),
                layer: Some(Layer::Character),
            },
            player_data,
        );
        let boat_data = entity_data! {
            boat: Boat::new(Radians(0f64)),
        };
        world.insert_entity_data(
            Location {
                coord: Coord::new(50, 50),
                layer: None,
            },
            boat_data,
        );
        Self::text_island(Coord::new(65, 40), &mut world);
        let water_visible_chance = 0.01f64;
        for coord in world.spatial_table.grid_size().coord_iter_row_major() {
            if world.spatial_table.layers_at_checked(coord).floor.is_none() {
                if rng.gen::<f64>() < water_visible_chance {
                    world.spawn_water1(coord);
                } else {
                    world.spawn_water2(coord);
                }
            }
        }
        Self {
            world,
            player_entity,
        }
    }
}
