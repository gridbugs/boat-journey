use crate::{
    world::{
        data::{Boat, EntityData, Junk, Npc},
        spatial::{Layer, Layers, Location},
        World,
    },
    Entity,
};
use coord_2d::{Coord, Size};
use entity_table::entity_data;
use procgen::{
    generate, generate_dungeon, Dungeon as DungeonGen, DungeonCell, Spec, WaterType, WorldCell3,
};
use rand::{seq::SliceRandom, Rng};
use serde::{Deserialize, Serialize};

pub struct Terrain {
    pub world: World,
    pub player_entity: Entity,
    pub num_dungeons: usize,
}

impl Terrain {
    pub fn generate<R: Rng>(
        player_data: EntityData,
        mut victories: Vec<crate::Victory>,
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
                                world.spawn_floor(coord);
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
                                    world.spawn_floor(coord);
                                } else {
                                    world.spawn_floor(coord);
                                }
                            } else {
                                if rng.gen::<f64>() < tree_chance1 {
                                    world.spawn_tree(coord);
                                } else if rng.gen::<f64>() < rock_chance2 {
                                    world.spawn_floor(coord);
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
                    WorldCell3::Grave => {
                        if let Some(victory) = victories.pop() {
                            world.spawn_grave(coord, victory);
                        } else {
                            world.spawn_floor(coord);
                        }
                    }
                    WorldCell3::Gate => {
                        if rng.gen::<f64>() < water_visible_chance {
                            world.spawn_water1(coord);
                        } else {
                            world.spawn_water2(coord);
                        }
                        world.spawn_gate(coord);
                    }
                }
            }
        }
        for &coord in g.world3.unimportant_npc_spawns.iter() {
            world.spawn_unimportant_npc(coord);
        }
        let mut all_npcs = Npc::all();
        all_npcs.shuffle(rng);
        for &coord in g.world3.npc_spawns.iter() {
            if let Some(npc) = all_npcs.pop() {
                world.spawn_npc(coord, npc);
            }
        }
        let all_junk = Junk::all();
        for &coord in &g.world3.junk_spawns {
            world.spawn_junk(coord, *all_junk.choose(rng).unwrap());
        }
        for &coord in &g.world3.inside_coords {
            if let Layers {
                floor: Some(floor), ..
            } = world.spatial_table.layers_at_checked(coord)
            {
                world.components.inside.insert(*floor, ());
            }
        }
        for (i, &coord) in g.world3.shop_coords.iter().enumerate() {
            world.spawn_shop(coord, i);
        }
        let mut island_coords = g
            .world3
            .island_coords
            .iter()
            .cloned()
            .filter(|&c| world.spatial_table.layers_at_checked(c).feature.is_none())
            .collect::<Vec<_>>();
        island_coords.shuffle(rng);
        for _ in 0..15 {
            if let Some(c) = island_coords.pop() {
                world.spawn_junk(c, *all_junk.choose(rng).unwrap());
            }
        }
        for _ in 0..20 {
            if let Some(c) = island_coords.pop() {
                world.spawn_beast(c);
            }
        }
        let mut building_coords = g
            .world3
            .building_coords
            .iter()
            .cloned()
            .filter(|&c| world.spatial_table.layers_at_checked(c).feature.is_none())
            .collect::<Vec<_>>();
        for _ in 0..15 {
            if let Some(c) = building_coords.pop() {
                world.spawn_junk(c, *all_junk.choose(rng).unwrap());
            }
        }
        for _ in 0..20 {
            if let Some(c) = building_coords.pop() {
                world.spawn_beast(c);
            }
        }
        building_coords.shuffle(rng);
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
        let DungeonGen {
            grid,
            spawn,
            destination,
            mut other_room_centres,
        } = generate_dungeon(size, rng);
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
        world.spawn_button(destination);
        let num_beasts = 3;
        other_room_centres.shuffle(rng);
        for _ in 0..num_beasts {
            if let Some(coord) = other_room_centres.pop() {
                world.spawn_beast(coord);
            }
        }
        let num_junk = 3;
        let all_junk = Junk::all();
        for _ in 0..num_junk {
            if let Some(coord) = other_room_centres.pop() {
                world.spawn_junk(coord, *all_junk.choose(rng).unwrap());
            }
        }
        Self { world, spawn }
    }
}
