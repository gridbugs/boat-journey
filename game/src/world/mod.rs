use gridbugs::{coord_2d::Size, entity_table::EntityAllocator};
use serde::{Deserialize, Serialize};

pub mod spatial;
use spatial::SpatialTable;

mod data;
use data::Components;
pub use data::Tile;

pub mod spawn;

#[derive(Debug, Serialize, Deserialize)]
pub struct World {
    pub entity_allocator: EntityAllocator,
    pub components: Components,
    pub spatial_table: SpatialTable,
}

impl World {
    pub fn new(size: Size) -> Self {
        let entity_allocator = EntityAllocator::default();
        let components = Components::default();
        let spatial_table = SpatialTable::new(size);
        Self {
            entity_allocator,
            components,
            spatial_table,
        }
    }
}
