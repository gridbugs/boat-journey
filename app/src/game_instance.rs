use crate::{game, stars::Stars, ui};
use chargrid::{prelude::*, text::StyledString};
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
        self.render_floor_text(ctx, fb);
        self.render_message_log(ctx, fb);
        self.render_hud(ctx, fb);
    }

    fn floor_text(&self) -> StyledString {
        let current_floor = self.game.inner_ref().current_level();
        let final_floor = orbital_decay_game::FINAL_LEVEL;
        if current_floor == 0 {
            StyledString {
                style: Style::new()
                    .with_foreground(Rgba32::new_grey(255))
                    .with_bold(true),
                string: format!(
                    "Gotta get to the fuel bay on the {}th floor...",
                    final_floor
                ),
            }
        } else if current_floor == final_floor {
            StyledString {
                style: Style::new().with_foreground(Rgba32::new_grey(255)),
                string: format!(
                    "Gotta get to the fuel bay on the {}th floor...",
                    final_floor
                ),
            }
        } else {
            StyledString {
                style: Style::new()
                    .with_foreground(Rgba32::new_grey(255))
                    .with_bold(true),
                string: format!("Floor {}/{}", current_floor, final_floor),
            }
        }
    }

    fn render_floor_text(&self, ctx: Ctx, fb: &mut FrameBuffer) {
        self.floor_text().render(&(), ctx, fb);
    }

    fn render_message_log(&self, ctx: Ctx, fb: &mut FrameBuffer) {
        ui::render_message_log(
            self.game.inner_ref().message_log(),
            ctx.add_offset(Coord { x: 1, y: 46 }),
            fb,
        );
    }

    fn render_hud(&self, ctx: Ctx, fb: &mut FrameBuffer) {
        let player = self.game.inner_ref().player();
        let player_info = self.game.inner_ref().player_info();
        ui::render_hud(player, player_info, ctx.add_xy(64, 4), fb);
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
