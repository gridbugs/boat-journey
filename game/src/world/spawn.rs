use crate::{
    world::{
        data::{EntityData, Location, Tile},
        World,
    },
    Entity,
};

pub fn make_player() -> EntityData {
    EntityData {
        tile: Some(Tile::Player),
        ..Default::default()
    }
}

impl World {
    pub fn insert_entity_data(&mut self, location: Location, entity_data: EntityData) -> Entity {
        let entity = self.entity_allocator.alloc();
        self.spatial_table.update(entity, location).unwrap();
        self.components.insert_entity_data(entity, entity_data);
        entity
    }
}
