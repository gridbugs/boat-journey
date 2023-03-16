use coord_2d::Size;
use entity_table::EntityAllocator;
use grid_search_cardinal_distance_map::DistanceMap;
use serde::{Deserialize, Serialize};

pub mod spatial;
use spatial::SpatialTable;

pub mod data;
use data::Components;

pub mod spawn;

#[derive(Debug, Serialize, Deserialize)]
pub struct World {
    pub entity_allocator: EntityAllocator,
    pub components: Components,
    pub spatial_table: SpatialTable,
    pub distance_map: DistanceMap,
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
            distance_map: DistanceMap::new(size),
        }
    }
}
