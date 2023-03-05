pub use crate::world::spatial::{Layer, Location};
use gridbugs::{coord_2d::Coord, entity_table::declare_entity_module};
use serde::{Deserialize, Serialize};
use vector::Radians;

declare_entity_module! {
    components {
        tile: Tile,
        boat: Boat,
    }
}
pub use components::Components;
pub use components::EntityData;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Tile {
    Player,
    BoatEdge,
    BoatFloor,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Boat {
    pub heading: Radians,
}

impl Boat {
    pub fn new(heading: Radians) -> Self {
        Self { heading }
    }
}
