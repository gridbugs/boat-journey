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
    pub fn generate<R: Rng>(player_data: EntityData, rng: &mut R) -> Self {
        let mut world = World::new(Size::new(100, 100));
        let player_entity = world.insert_entity_data(
            Location {
                coord: Coord::new(20, 20),
                layer: Some(Layer::Character),
            },
            player_data,
        );
        let boat_data = entity_data! {
            boat: Boat::new(Radians(0f64)),
        };
        world.insert_entity_data(
            Location {
                coord: Coord::new(20, 20),
                layer: None,
            },
            boat_data,
        );
        Self {
            world,
            player_entity,
        }
    }
}
