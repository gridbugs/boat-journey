use crate::{game, stars::Stars};
use chargrid::prelude::*;
use orbital_decay_game::{
    witness::{self, Game, RunningGame},
    Config, Music,
};
use rand::Rng;
use serde::{Deserialize, Serialize};

pub struct GameInstance {
    pub game: Game,
    stars: Stars,
    pub current_music: Option<Music>,
}

impl GameInstance {
    pub fn new<R: Rng>(config: &Config, rng: &mut R) -> (Self, witness::Running) {
        let (game, running) = witness::new_game(config, rng);
        let stars = Stars::new(rng);
        (
            GameInstance {
                game,
                stars,
                current_music: None,
            },
            running,
        )
    }

    pub fn into_storable(self, running: witness::Running) -> GameInstanceStorable {
        let Self {
            game,
            stars,
            current_music,
        } = self;
        let running_game = game.into_running_game(running);
        GameInstanceStorable {
            running_game,
            stars,
            current_music,
        }
    }

    pub fn render(&self, ctx: Ctx, fb: &mut FrameBuffer) {
        self.stars
            .render_with_visibility(self.game.inner_ref().visibility_grid(), ctx, fb);
        game::render_game_with_visibility(self.game.inner_ref(), ctx, fb);
    }
}

#[derive(Serialize, Deserialize)]
pub struct GameInstanceStorable {
    running_game: RunningGame,
    stars: Stars,
    current_music: Option<Music>,
}

impl GameInstanceStorable {
    pub fn into_game_instance(self) -> (GameInstance, witness::Running) {
        let Self {
            running_game,
            stars,
            current_music,
        } = self;
        let (game, running) = running_game.into_game();
        (
            GameInstance {
                game,
                stars,
                current_music,
            },
            running,
        )
    }
}
