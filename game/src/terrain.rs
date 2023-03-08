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
use procgen::{generate, Spec, WaterType, WorldCell2, WorldCell3};
use rand::Rng;
use serde::{Deserialize, Serialize};
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

    pub fn generate_debug<R: Rng>(player_data: EntityData, rng: &mut R) -> Self {
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

    pub fn generate<R: Rng>(player_data: EntityData, rng: &mut R) -> Self {
        let g = generate(
            &Spec {
                size: Size::new(150, 80),
            },
            rng,
        );
        let mut world = World::new(g.world3.grid.size());
        let player_entity = world.insert_entity_data(
            Location {
                coord: g.world3.spawn,
                layer: Some(Layer::Character),
            },
            player_data,
        );
        let boat_data = entity_data! {
            boat: Boat::new(g.world3.boat_heading),
        };
        world.insert_entity_data(
            Location {
                coord: g.world3.boat_spawn,
                layer: None,
            },
            boat_data,
        );
        let water_visible_chance = 0.01f64;
        let ocean_water_visible_chance = 0.2f64;
        let tree_chance1 = 0.2f64;
        let tree_chance2 = 0.4f64;
        let tree_chance3 = 0.05f64;
        let rock_chance1 = 0.05f64;
        let rock_chance2 = 0.1f64;
        let mut num_stairs = 0;
        for (coord, &cell) in g.world3.grid.enumerate() {
            let water_distance = *g.water_distance_map.distances.get_checked(coord);
            if water_distance < 20 {
                match cell {
                    WorldCell3::Ground => {
                        if coord.x > g.world2.ocean_x_ofset as i32 - 5 {
                            if rng.gen::<f64>() < rock_chance1 {
                                world.spawn_rock(coord);
                            } else {
                                world.spawn_floor(coord);
                            }
                        } else {
                            if water_distance > 15 {
                                world.spawn_tree(coord);
                            } else if water_distance > 7 {
                                if rng.gen::<f64>() < tree_chance2 {
                                    world.spawn_tree(coord);
                                } else if rng.gen::<f64>() < rock_chance1 {
                                    world.spawn_rock(coord);
                                } else {
                                    world.spawn_floor(coord);
                                }
                            } else {
                                if rng.gen::<f64>() < tree_chance1 {
                                    world.spawn_tree(coord);
                                } else if rng.gen::<f64>() < rock_chance2 {
                                    world.spawn_rock(coord);
                                } else {
                                    world.spawn_floor(coord);
                                }
                            }
                        }
                    }
                    WorldCell3::Water(WaterType::River) => {
                        if rng.gen::<f64>() < water_visible_chance {
                            world.spawn_water1(coord);
                        } else {
                            world.spawn_water2(coord);
                        }
                    }
                    WorldCell3::Water(WaterType::Ocean) => {
                        if rng.gen::<f64>() < ocean_water_visible_chance {
                            world.spawn_ocean_water1(coord);
                        } else {
                            world.spawn_ocean_water2(coord);
                        }
                    }
                    WorldCell3::Door => {
                        world.spawn_door(coord);
                    }
                    WorldCell3::Floor => {
                        world.spawn_floor(coord);
                    }
                    WorldCell3::TownGround => {
                        if rng.gen::<f64>() < tree_chance3 {
                            world.spawn_tree(coord);
                        } else {
                            world.spawn_floor(coord);
                        }
                    }
                    WorldCell3::Wall => {
                        world.spawn_wall(coord);
                    }
                    WorldCell3::StairsDown => {
                        num_stairs += 1;
                        world.spawn_stairs_down(coord, num_stairs);
                    }
                    WorldCell3::StairsUp => {
                        world.spawn_stairs_up(coord);
                    }
                }
            }
        }
        Self {
            world,
            player_entity,
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct Dungeon {
    pub world: World,
    pub spawn: Coord,
}

impl Dungeon {
    pub fn generate<R: Rng>(rng: &mut R) -> Self {
        let size = Size::new(10, 10);
        let mut world = World::new(size);
        for coord in size.coord_iter_row_major() {
            world.spawn_floor(coord);
        }
        let spawn = Coord::new(1, 2);
        world.spawn_stairs_up(spawn);
        Self { world, spawn }
    }
}
