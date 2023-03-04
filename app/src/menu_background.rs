use crate::{game, stars::Stars};
use gridbugs::chargrid::prelude::*;
use template2023_game::{Config, Game, Omniscient};
use rand::Rng;
use std::time::Duration;

pub struct MenuBackground {
    game: Game,
    stars: Stars,
    duration: Duration,
}

impl MenuBackground {
    pub fn new<R: Rng>(rng: &mut R) -> Self {
        let config = Config {
            omniscient: Some(Omniscient),
            demo: true,
            debug: false,
        };
        let game = Game::new(&config, rng);
        let stars = Stars::new(rng);
        let duration = Duration::from_millis(0);
        Self {
            game,
            stars,
            duration,
        }
    }

    pub fn render_stars(&self, ctx: Ctx, fb: &mut FrameBuffer) {
        self.stars.render(ctx, fb);
    }

    pub fn render(&self, ctx: Ctx, fb: &mut FrameBuffer) {
        self.stars.render(ctx, fb);
        let ctx = ctx.add_offset(Coord { x: 38, y: 5 });
        game::render_game(&self.game, ctx, fb);
    }

    pub fn tick(&mut self, since_prev: Duration) {
        const NPC_TURN_PERIOD: Duration = Duration::from_millis(2000);
        self.duration += since_prev;
        if self.duration > NPC_TURN_PERIOD {
            self.duration -= NPC_TURN_PERIOD;
            self.game.handle_npc_turn();
        }
    }
}
