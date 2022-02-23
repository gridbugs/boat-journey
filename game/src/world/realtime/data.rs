use crate::world::realtime::{
    fade::FadeState, flicker::FlickerState, light_colour_fade::LightColourFadeState,
    movement::MovementState, particle::ParticleEmitterState, Context,
};
use entity_table_realtime::declare_realtime_entity_module;

declare_realtime_entity_module! {
    components<'a>[Context<'a>] {
        movement: MovementState,
        fade: FadeState,
        light_colour_fade: LightColourFadeState,
        particle_emitter: ParticleEmitterState,
        flicker: FlickerState,
    }
}
pub use components::RealtimeComponents;
