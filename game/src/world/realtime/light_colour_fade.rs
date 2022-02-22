use crate::world::realtime::{
    fade::{FadeProgress, FadeState},
    Context,
};
use entity_table_realtime::{Entity, RealtimeComponent, RealtimeComponentApplyEvent};
use rgb_int::Rgb24;
use serde::{Deserialize, Serialize};
use std::time::Duration;

pub enum LightColourFadeProgress {
    Colour(Rgb24),
    Complete,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LightColourFadeState {
    pub fade_state: FadeState,
    pub from: Rgb24,
    pub to: Rgb24,
}

impl RealtimeComponent for LightColourFadeState {
    type Event = LightColourFadeProgress;
    fn tick(&mut self) -> (Self::Event, Duration) {
        let (fade_progress, until_next_event) = self.fade_state.tick();
        let event = match fade_progress {
            FadeProgress::Complete => LightColourFadeProgress::Complete,
            FadeProgress::Fading(fading) => {
                LightColourFadeProgress::Colour(self.from.linear_interpolate(self.to, fading))
            }
        };
        (event, until_next_event)
    }
}

impl<'a> RealtimeComponentApplyEvent<Context<'a>> for LightColourFadeState {
    fn apply_event(progress: LightColourFadeProgress, entity: Entity, context: &mut Context<'a>) {
        match progress {
            LightColourFadeProgress::Colour(colour) => {
                if let Some(light) = context.world.components.light.get_mut(entity) {
                    light.colour = colour;
                }
            }
            LightColourFadeProgress::Complete => {
                context.world.components.light.remove(entity);
            }
        }
    }
}
