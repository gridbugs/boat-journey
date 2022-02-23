use crate::world::realtime::{
    animation::FRAME_DURATION, fade::FadeState, light_colour_fade::LightColourFadeState,
    movement::MovementState, particle::ParticleEmitterState, Context,
};
use entity_table_realtime::declare_realtime_entity_module;
use std::time::Duration;

pub fn period_per_frame(num_per_frame: u32) -> Duration {
    FRAME_DURATION / num_per_frame
}

declare_realtime_entity_module! {
    components<'a>[Context<'a>] {
        movement: MovementState,
        fade: FadeState,
        light_colour_fade: LightColourFadeState,
        particle_emitter: ParticleEmitterState,
    }
}
pub use components::RealtimeComponents;
