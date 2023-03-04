pub use crate::world::spatial::{Layer, Location};
use gridbugs::{
    coord_2d::Axis, direction::CardinalDirection, entity_table::declare_entity_module,
    rgb_int::Rgb24,
};
use serde::{Deserialize, Serialize};

declare_entity_module! {
    components {
        tile: Tile,
    }
}
pub use components::Components;
pub use components::EntityData;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Tile {
    Player,
}
