use crate::world::realtime::Context;
use gridbugs::entity_table_realtime::{Entity, RealtimeComponent, RealtimeComponentApplyEvent};
use serde::{Deserialize, Serialize};
use std::time::Duration;

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum FadeProgress {
    Fading(u8),
    Complete,
}

impl FadeProgress {
    fn fading(self) -> Option<u8> {
        match self {
            Self::Fading(progress) => Some(progress),
            Self::Complete => None,
        }
    }
    fn is_complete(self) -> bool {
        match self {
            Self::Fading(_) => false,
            Self::Complete => true,
        }
    }
}

impl Default for FadeProgress {
    fn default() -> Self {
        Self::Fading(0)
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct FadeState {
    progress: FadeProgress,
    period: Duration,
}

impl FadeState {
    pub fn new(duration: Duration) -> Self {
        Self::new_with_progress(duration, FadeProgress::default())
    }
    pub fn new_with_progress(full_duration: Duration, progress: FadeProgress) -> Self {
        let period = full_duration / 256;
        Self { progress, period }
    }
    pub fn fading(self) -> Option<u8> {
        self.progress.fading()
    }
}

impl RealtimeComponent for FadeState {
    type Event = FadeProgress;

    fn tick(&mut self) -> (Self::Event, Duration) {
        self.progress = match self.progress {
            FadeProgress::Complete => FadeProgress::Complete,
            FadeProgress::Fading(progress) => match progress.checked_add(1) {
                Some(progress) => FadeProgress::Fading(progress),
                None => FadeProgress::Complete,
            },
        };
        (self.progress, self.period)
    }
}

impl<'a> RealtimeComponentApplyEvent<Context<'a>> for FadeState {
    fn apply_event(progress: FadeProgress, entity: Entity, context: &mut Context<'a>) {
        if progress.is_complete() {
            context.world.entity_allocator.free(entity);
            context.world.components.remove_entity(entity);
        }
    }
}
