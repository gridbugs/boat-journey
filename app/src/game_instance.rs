use gridbugs::chargrid::{prelude::*, text::StyledString};
use rand::Rng;
use serde::{Deserialize, Serialize};
use template2023_game::{
    witness::{self, Game, RunningGame},
    Config,
};

pub struct GameInstance {
    pub game: Game,
}

impl GameInstance {
    pub fn new<R: Rng>(config: &Config, rng: &mut R) -> (Self, witness::Running) {
        let (game, running) = witness::new_game(config, rng);
        (GameInstance { game }, running)
    }

    pub fn into_storable(self, running: witness::Running) -> GameInstanceStorable {
        let Self { game } = self;
        let running_game = game.into_running_game(running);
        GameInstanceStorable { running_game }
    }

    pub fn render(&self, ctx: Ctx, fb: &mut FrameBuffer) {}
}

#[derive(Serialize, Deserialize)]
pub struct GameInstanceStorable {
    running_game: RunningGame,
}

impl GameInstanceStorable {
    pub fn into_game_instance(self) -> (GameInstance, witness::Running) {
        let Self { running_game } = self;
        let (game, running) = running_game.into_game();
        (GameInstance { game }, running)
    }
}
