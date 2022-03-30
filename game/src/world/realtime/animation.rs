use crate::{
    world::{realtime::Context, World},
    ExternalEvent, Message,
};
use gridbugs::{entity_table::Entity, entity_table_realtime};
use rand_isaac::Isaac64Rng;
use serde::{Deserialize, Serialize};
use std::time::Duration;

pub const FRAME_DURATION: Duration = Duration::from_micros(1_000_000 / 60);

#[derive(Default)]
pub struct AnimationContext {
    realtime_entities: Vec<Entity>,
}

impl Serialize for AnimationContext {
    fn serialize<S: serde::Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        ().serialize(s)
    }
}

impl<'a> Deserialize<'a> for AnimationContext {
    fn deserialize<D: serde::Deserializer<'a>>(d: D) -> Result<Self, D::Error> {
        let () = Deserialize::deserialize(d)?;
        Ok(Self::default())
    }
}

impl AnimationContext {
    pub fn tick(
        &mut self,
        world: &mut World,
        external_events: &mut Vec<ExternalEvent>,
        message_log: &mut Vec<Message>,
        rng: &mut Isaac64Rng,
    ) {
        self.realtime_entities
            .extend(world.components.realtime.entities());
        let mut context = Context {
            world,
            external_events,
            message_log,
            rng,
        };
        for entity in self.realtime_entities.drain(..) {
            entity_table_realtime::process_entity_frame(entity, FRAME_DURATION, &mut context);
        }
    }
}
