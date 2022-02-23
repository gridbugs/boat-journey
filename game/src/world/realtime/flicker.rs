use crate::world::{realtime::Context, Tile};
use entity_table_realtime::{Entity, RealtimeComponent, RealtimeComponentApplyEvent};
use rand::{seq::SliceRandom, Rng, SeedableRng};
use rand_isaac::Isaac64Rng;
use rgb_int::Rgb24;
use serde::{Deserialize, Serialize};
use std::time::Duration;

pub mod spec {
    pub use crate::world::Tile;
    pub use rand_range::UniformInclusiveRange;
    pub use rgb_int::Rgb24;
    use serde::{Deserialize, Serialize};
    pub use std::time::Duration;

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct Flicker {
        pub colour_hint: Option<UniformInclusiveRange<Rgb24>>,
        pub light_colour: Option<UniformInclusiveRange<Rgb24>>,
        pub tile: Option<Vec<Tile>>,
        pub until_next_event: UniformInclusiveRange<Duration>,
    }
}

impl spec::Flicker {
    pub fn build<R: Rng>(self, rng: &mut R) -> FlickerState {
        FlickerState {
            flicker: self,
            rng: Isaac64Rng::from_rng(rng).unwrap(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlickerState {
    flicker: spec::Flicker,
    rng: Isaac64Rng,
}

pub struct FlickerEvent {
    colour_hint: Option<Rgb24>,
    light_colour: Option<Rgb24>,
    tile: Option<Tile>,
}

impl RealtimeComponent for FlickerState {
    type Event = FlickerEvent;

    fn tick(&mut self) -> (Self::Event, Duration) {
        let colour_hint = self.flicker.colour_hint.map(|r| r.choose(&mut self.rng));
        let light_colour = self.flicker.light_colour.map(|r| r.choose(&mut self.rng));
        let tile = self
            .flicker
            .tile
            .as_ref()
            .and_then(|t| t.choose(&mut self.rng))
            .cloned();
        let until_next_event = self.flicker.until_next_event.choose(&mut self.rng);
        let event = FlickerEvent {
            colour_hint,
            light_colour,
            tile,
        };
        (event, until_next_event)
    }
}

impl<'a> RealtimeComponentApplyEvent<Context<'a>> for FlickerState {
    fn apply_event(event: FlickerEvent, entity: Entity, context: &mut Context<'a>) {
        if let Some(colour_hint) = event.colour_hint {
            context
                .world
                .components
                .colour_hint
                .insert(entity, colour_hint);
        }
        if let Some(light_colour) = event.light_colour {
            if let Some(light) = context.world.components.light.get_mut(entity) {
                light.colour = light_colour;
            }
        }
        if let Some(tile) = event.tile {
            context.world.components.tile.insert(entity, tile);
        }
    }
}
