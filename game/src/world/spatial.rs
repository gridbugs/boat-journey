use gridbugs::spatial_table;
spatial_table::declare_layers_module! {
    layers {
        floor: Floor,
        feature: Feature,
        character: Character,
        item: Item,
        boat: Boat,
        water: Water,
    }
}
pub use layers::{Layer, Layers};
pub type SpatialTable = spatial_table::SpatialTable<Layers>;
pub type Location = spatial_table::Location<Layer>;
pub use spatial_table::UpdateError;
