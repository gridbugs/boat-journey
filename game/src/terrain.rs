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
use procgen::{
    generate, generate_dungeon, Dungeon as DungeonGen, DungeonCell, Spec, WaterType, WorldCell2,
    WorldCell3,
};
use rand::Rng;
use serde::{Deserialize, Serialize};
use vector::Radians;

pub struct Terrain {
    pub world: World,
    pub player_entity: Entity,
    pub num_dungeons: usize,
}

impl Terrain {
    pub fn generate<R: Rng>(
        player_data: EntityData,
        victories: Vec<crate::Victory>,
        rng: &mut R,
    ) -> Self {
        let g = generate(
            &Spec {
                size: Size::new(150, 80),
                num_graves: victories.len() as u32,
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
                        if coord == g.world3.your_door {
                            world.spawn_player_door(coord);
                        } else {
                            world.spawn_door(coord);
                        }
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
        for &coord in g.world3.unimportant_npc_spawns.iter() {
            world.spawn_unimportant_npc(coord);
        }
        Self {
            world,
            player_entity,
            num_dungeons: num_stairs,
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
        let size = Size::new(30, 30);
        let mut world = World::new(size);
        let DungeonGen { grid, spawn } = generate_dungeon(size, rng);
        for (coord, &cell) in grid.enumerate() {
            match cell {
                DungeonCell::Door => {
                    world.spawn_door(coord);
                }
                DungeonCell::Wall => {
                    world.spawn_wall(coord);
                }
                DungeonCell::Floor => {
                    world.spawn_floor(coord);
                }
            }
        }
        world.spawn_stairs_up(spawn);
        Self { world, spawn }
    }
}
