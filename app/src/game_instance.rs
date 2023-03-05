use boat_journey_game::{
    witness::{self, Game, RunningGame},
    Config, Layer, Tile,
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

    fn layer_to_depth(layer: Layer) -> i8 {
        match layer {
            Layer::Character => 5,
            Layer::Item => 4,
            Layer::Feature => 3,
            Layer::Boat => 2,
            Layer::Floor => 1,
            Layer::Water => 0,
        }
    }

    fn tile_to_render_cell(tile: Tile) -> RenderCell {
        let character = match tile {
            Tile::Player => '@',
            Tile::BoatEdge => '#',
            Tile::BoatFloor => ',',
            Tile::Water1 => '~',
            Tile::Water2 => ' ',
            Tile::Floor => '.',
            Tile::Wall => '█',
            Tile::DoorClosed => '+',
            Tile::DoorOpen => '-',
            Tile::Rock => '░',
            Tile::Board => '=',
        };
        RenderCell {
            character: Some(character),
            style: Style::new()
                .with_foreground(Rgba32::new_grey(187))
                .with_background(Rgba32::new_rgb(0, 0x40, 0x40)),
        }
    }

    pub fn render(&self, ctx: Ctx, fb: &mut FrameBuffer) {
        let centre_coord_delta =
            self.game.inner_ref().player_coord() - (ctx.bounding_box.size() / 2);
        self.game.inner_ref().render(|location, tile| {
            if let Some(layer) = location.layer {
                let depth = Self::layer_to_depth(layer);
                let render_cell = Self::tile_to_render_cell(tile);
                fb.set_cell_relative_to_ctx(
                    ctx,
                    location.coord - centre_coord_delta,
                    depth,
                    render_cell,
                );
            }
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
