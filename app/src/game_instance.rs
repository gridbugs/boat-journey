use crate::colour;
use boat_journey_game::{
    witness::{self, Game, RunningGame},
    Config, Layer, Tile,
};
use gridbugs::{chargrid::prelude::*, visible_area_detection::CellVisibility};
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

    fn tile_to_render_cell(tile: Tile, current: bool) -> RenderCell {
        let character = match tile {
            Tile::Player => {
                return RenderCell {
                    character: Some('@'),
                    style: Style::new()
                        .with_bold(true)
                        .with_foreground(Rgba32::new_grey(255))
                        .with_background(colour::MURKY_GREEN.to_rgba32(255)),
                };
            }
            Tile::BoatControls => {
                return RenderCell {
                    character: Some('░'),
                    style: Style::new()
                        .with_bold(true)
                        .with_foreground(Rgba32::new_grey(255))
                        .with_background(colour::MURKY_GREEN.to_rgba32(255)),
                };
            }
            Tile::BoatEdge => '#',
            Tile::BoatFloor => ',',
            Tile::Water1 => {
                if current {
                    '~'
                } else {
                    ' '
                }
            }
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
                .with_bold(false)
                .with_foreground(Rgba32::new_grey(187))
                .with_background(colour::MURKY_GREEN.to_rgba32(255)),
        }
    }

    pub fn render(&self, ctx: Ctx, fb: &mut FrameBuffer) {
        let centre_coord_delta =
            self.game.inner_ref().player_coord() - (ctx.bounding_box.size() / 2);
        for coord in ctx.bounding_box.size().coord_iter_row_major() {
            let cell = self
                .game
                .inner_ref()
                .cell_visibility_at_coord(coord + centre_coord_delta);
            match cell {
                CellVisibility::Never => {
                    let render_cell = RenderCell {
                        character: None,
                        style: Style::new().with_background(colour::MISTY_GREY.to_rgba32(255)),
                    };
                    fb.set_cell_relative_to_ctx(ctx, coord, 0, render_cell);
                }
                CellVisibility::Previous(data) => {
                    data.tiles.for_each_enumerate(|tile, layer| {
                        if let Some(&tile) = tile.as_ref() {
                            let depth = Self::layer_to_depth(layer);
                            let mut render_cell = Self::tile_to_render_cell(tile, false);
                            render_cell.style.background = Some(colour::MISTY_GREY.to_rgba32(255));
                            render_cell.style.foreground = Some(Rgba32::new_grey(63));
                            fb.set_cell_relative_to_ctx(ctx, coord, depth, render_cell);
                        }
                    });
                }
                CellVisibility::Current { data, .. } => {
                    data.tiles.for_each_enumerate(|tile, layer| {
                        if let Some(&tile) = tile.as_ref() {
                            let depth = Self::layer_to_depth(layer);
                            let render_cell = Self::tile_to_render_cell(tile, true);
                            fb.set_cell_relative_to_ctx(ctx, coord, depth, render_cell);
                        }
                    });
                }
            }
        }
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
