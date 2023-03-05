use boat_journey_game::{
    witness::{self, Game, RunningGame},
    Config, Tile,
};
use gridbugs::chargrid::prelude::*;
use rand::Rng;
use serde::{Deserialize, Serialize};

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

    fn tile_to_render_cell(tile: Tile) -> RenderCell {
        let character = match tile {
            Tile::Player => '@',
            Tile::BoatEdge => '#',
            Tile::BoatFloor => '.',
        };
        RenderCell {
            character: Some(character),
            style: Style::plain_text(),
        }
    }

    pub fn render(&self, ctx: Ctx, fb: &mut FrameBuffer) {
        self.game.inner_ref().render(|coord, tile| {
            fb.set_cell_relative_to_ctx(ctx, coord, 0, Self::tile_to_render_cell(tile));
        });
    }
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
