pub use crate::world::spatial::{Layer, Location};
use gridbugs::entity_table::declare_entity_module;
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
