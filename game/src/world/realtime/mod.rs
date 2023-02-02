use crate::{world::World, ExternalEvent, Message};
use gridbugs::entity_table_realtime::{ContextContainsRealtimeComponents, Entities};
use rand_isaac::Isaac64Rng;

pub mod animation;
pub mod data;
pub mod fade;
pub mod light_colour_fade;
pub mod movement;
pub mod particle;

pub struct Context<'a> {
    world: &'a mut World,
    external_events: &'a mut Vec<ExternalEvent>,
    message_log: &'a mut Vec<Message>,
    rng: &'a mut Isaac64Rng,
}

impl<'a> ContextContainsRealtimeComponents for Context<'a> {
    type Components = data::RealtimeComponents;
    fn components_mut(&mut self) -> &mut Self::Components {
        &mut self.world.realtime_components
    }
    fn realtime_entities(&self) -> Entities {
        self.world.components.realtime.entities()
    }
}
