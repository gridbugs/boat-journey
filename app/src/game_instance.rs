use crate::{colour, mist::Mist};
use boat_journey_game::{
    witness::{self, Game, RunningGame},
    Config, Layer, Meter, Npc, Tile, Victory,
};
use gridbugs::{
    chargrid::{prelude::*, text},
    rgb_int::{rgb24, Rgb24},
    visible_area_detection::CellVisibility,
};
use rand::Rng;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

#[derive(Serialize, Deserialize)]
pub struct FadeState {
    pub boat_opacity: u8,
    pub player_opacity: u8,
    pub boat_fading: bool,
    pub player_fading: bool,
}

impl FadeState {
    pub fn new() -> Self {
        Self {
            boat_opacity: 255,
            player_opacity: 255,
            boat_fading: false,
            player_fading: false,
        }
    }
}

struct NightTint;
impl Tint for NightTint {
    fn tint(&self, rgba32: Rgba32) -> Rgba32 {
        let mean = rgba32
            .to_rgb24()
            .weighted_mean_u16(rgb24::WeightsU16::new(1, 1, 1));
        Rgb24::new_grey(mean)
            .saturating_scalar_mul_div(3, 4)
            .to_rgba32(255)
    }
}

pub struct GameInstance {
    pub game: Game,
    pub mist: Mist,
    pub fade_state: FadeState,
}

fn npc_colour(npc: Npc) -> Rgb24 {
    let hex = match npc {
        Npc::Physicist => 0x5bcdcd, // cyan
        Npc::Soldier => 0x628139,
    };
    Rgb24::hex(hex)
}

impl GameInstance {
    pub fn new<R: Rng>(
        config: &Config,
        victories: Vec<Victory>,
        rng: &mut R,
    ) -> (Self, witness::Running) {
        let (game, running) = witness::new_game(config, victories, rng);
        let mist = Mist::new(rng);
        (
            GameInstance {
                game,
                mist,
                fade_state: FadeState::new(),
            },
            running,
        )
    }

    pub fn into_storable(self, running: witness::Running) -> GameInstanceStorable {
        let Self {
            game,
            mist,
            fade_state,
        } = self;
        let running_game = game.into_running_game(running);
        GameInstanceStorable {
            running_game,
            mist,
            fade_state,
        }
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

    fn tile_to_render_cell(
        tile: Tile,
        current: bool,
        boat_opacity: u8,
        player_opacity: u8,
    ) -> RenderCell {
        let character = match tile {
            Tile::Player => {
                return RenderCell {
                    character: Some('@'),
                    style: Style::new()
                        .with_bold(true)
                        .with_foreground(
                            Rgba32::new_grey(255)
                                .with_a(player_opacity)
                                .alpha_composite(colour::MURKY_GREEN.to_rgba32(255)),
                        )
                        .with_background(colour::MURKY_GREEN.to_rgba32(255)),
                };
            }
            Tile::BoatControls => {
                return RenderCell {
                    character: Some('░'),
                    style: Style::new()
                        .with_bold(true)
                        .with_foreground(
                            Rgba32::new_grey(255)
                                .with_a(boat_opacity)
                                .alpha_composite(colour::MURKY_GREEN.to_rgba32(255)),
                        )
                        .with_background(colour::MURKY_GREEN.to_rgba32(255)),
                };
            }
            Tile::BoatEdge => {
                return RenderCell {
                    character: Some('#'),
                    style: Style::new()
                        .with_bold(true)
                        .with_foreground(
                            Rgba32::new_grey(255)
                                .with_a(boat_opacity)
                                .alpha_composite(colour::MURKY_GREEN.to_rgba32(255)),
                        )
                        .with_background(colour::MURKY_GREEN.to_rgba32(255)),
                };
            }
            Tile::BoatFloor => {
                return RenderCell {
                    character: Some('.'),
                    style: Style::new()
                        .with_bold(true)
                        .with_foreground(
                            Rgba32::new_grey(255)
                                .with_a(boat_opacity)
                                .alpha_composite(colour::MURKY_GREEN.to_rgba32(255)),
                        )
                        .with_background(colour::MURKY_GREEN.to_rgba32(255)),
                };
            }
            Tile::Beast => {
                return RenderCell {
                    character: Some('b'),
                    style: Style::new()
                        .with_bold(true)
                        .with_foreground(
                            Rgba32::new_grey(255)
                                .with_a(255)
                                .alpha_composite(colour::MURKY_GREEN.to_rgba32(255)),
                        )
                        .with_background(colour::MURKY_GREEN.to_rgba32(255)),
                };
            }
            Tile::Ghost => {
                return RenderCell {
                    character: Some('g'),
                    style: Style::new()
                        .with_bold(true)
                        .with_foreground(
                            Rgba32::new_grey(255)
                                .with_a(255)
                                .alpha_composite(colour::MURKY_GREEN.to_rgba32(255)),
                        )
                        .with_background(colour::MURKY_GREEN.to_rgba32(255)),
                };
            }
            Tile::Grave => {
                return RenderCell {
                    character: Some('▄'),
                    style: Style::new()
                        .with_bold(true)
                        .with_foreground(
                            Rgba32::new_grey(255)
                                .with_a(255)
                                .alpha_composite(colour::MURKY_GREEN.to_rgba32(255)),
                        )
                        .with_background(colour::MURKY_GREEN.to_rgba32(255)),
                };
            }
            Tile::Water1 => {
                if current {
                    '~'
                } else {
                    ' '
                }
            }
            Tile::Water2 => ' ',
            Tile::Floor => '.',
            Tile::BurntFloor => {
                return RenderCell {
                    character: Some('.'),
                    style: Style::new()
                        .with_foreground(Rgba32::new_grey(63))
                        .with_background(Rgb24::new(0, 0, 0).to_rgba32(255)),
                }
            }
            Tile::Wall => '█',
            Tile::DoorClosed => '+',
            Tile::DoorOpen => '-',
            Tile::Rock => '%',
            Tile::Board => '=',
            Tile::Tree => '♣',
            Tile::UnimportantNpc => {
                return RenderCell {
                    character: Some('&'),
                    style: Style::new()
                        .with_foreground(Rgba32::new_grey(255))
                        .with_background(colour::MURKY_GREEN.to_rgba32(255)),
                };
            }
            Tile::StairsDown => {
                return RenderCell {
                    character: Some('>'),
                    style: Style::new()
                        .with_bold(true)
                        .with_foreground(Rgba32::new_grey(255))
                        .with_background(colour::MURKY_GREEN.to_rgba32(255)),
                };
            }
            Tile::StairsUp => {
                return RenderCell {
                    character: Some('<'),
                    style: Style::new()
                        .with_bold(true)
                        .with_foreground(Rgba32::new_grey(255))
                        .with_background(colour::MURKY_GREEN.to_rgba32(255)),
                };
            }
            Tile::Junk => {
                return RenderCell {
                    character: Some('*'),
                    style: Style::new()
                        .with_bold(true)
                        .with_foreground(Rgba32::new_grey(255))
                        .with_background(colour::MURKY_GREEN.to_rgba32(255)),
                };
            }
            Tile::Button => {
                return RenderCell {
                    character: Some('/'),
                    style: Style::new()
                        .with_bold(true)
                        .with_foreground(Rgba32::new_grey(255))
                        .with_background(colour::MURKY_GREEN.to_rgba32(255)),
                };
            }
            Tile::ButtonPressed => {
                return RenderCell {
                    character: Some('\\'),
                    style: Style::new()
                        .with_bold(true)
                        .with_foreground(Rgba32::new_grey(255))
                        .with_background(colour::MURKY_GREEN.to_rgba32(255)),
                };
            }
            Tile::Shop => {
                return RenderCell {
                    character: Some('$'),
                    style: Style::new()
                        .with_bold(true)
                        .with_foreground(Rgba32::new_grey(255))
                        .with_background(colour::MURKY_GREEN.to_rgba32(255)),
                };
            }
            Tile::Npc(_npc) => {
                return RenderCell {
                    character: Some('@'),
                    style: Style::plain_text()
                        .with_foreground(Rgb24::new_grey(255).to_rgba32(player_opacity))
                        //.with_foreground(npc_colour(npc).to_rgba32(player_opacity))
                        .with_background(colour::MURKY_GREEN.to_rgba32(255)),
                };
            }
        };
        RenderCell {
            character: Some(character),
            style: Style::new()
                .with_bold(false)
                .with_foreground(Rgba32::new_grey(187))
                .with_background(colour::MURKY_GREEN.to_rgba32(255)),
        }
    }

    pub fn render_game(&self, ctx: Ctx, fb: &mut FrameBuffer) -> HashSet<Tile> {
        let mut tiles = HashSet::new();
        let ctx = if self.game.inner_ref().is_player_outside_at_night() {
            ctx.with_tint(&NightTint)
        } else {
            ctx
        };
        let centre_coord_delta =
            self.game.inner_ref().player_coord() - (ctx.bounding_box.size() / 2);
        let boat_opacity = self.fade_state.boat_opacity;
        let player_opacity = self.fade_state.player_opacity;
        for coord in ctx.bounding_box.size().coord_iter_row_major() {
            let cell = self
                .game
                .inner_ref()
                .cell_visibility_at_coord(coord + centre_coord_delta);
            let mut mist = if self.game.inner_ref().is_in_dungeon() {
                Rgba32::new(0, 0, 0, 0)
            } else {
                self.mist.get(coord)
            };
            if self.game.inner_ref().is_player_outside_at_night() {
                mist.a *= 4;
            }

            let unseen_background = if self.game.inner_ref().is_in_dungeon() {
                Rgba32::new(0, 0, 0, 255)
            } else {
                colour::MISTY_GREY.to_rgba32(255)
            };

            match cell {
                CellVisibility::Never => {
                    let background = mist.alpha_composite(unseen_background);
                    let render_cell = RenderCell {
                        character: None,
                        style: Style::new().with_background(background),
                    };
                    fb.set_cell_relative_to_ctx(ctx, coord, 0, render_cell);
                }
                CellVisibility::Previous(data) => {
                    let background = mist.alpha_composite(unseen_background);
                    data.tiles.for_each_enumerate(|tile, layer| {
                        if let Some(&tile) = tile.as_ref() {
                            let depth = Self::layer_to_depth(layer);
                            let mut render_cell = Self::tile_to_render_cell(
                                tile,
                                false,
                                boat_opacity,
                                player_opacity,
                            );
                            render_cell.style.background = Some(background);
                            render_cell.style.foreground = Some(Rgba32::new_grey(63));
                            fb.set_cell_relative_to_ctx(ctx, coord, depth, render_cell);
                        }
                    });
                }
                CellVisibility::Current { data, .. } => {
                    data.tiles.for_each_enumerate(|tile, layer| {
                        if let Some(&tile) = tile.as_ref() {
                            tiles.insert(tile);
                            let depth = Self::layer_to_depth(layer);
                            let mut render_cell =
                                Self::tile_to_render_cell(tile, true, boat_opacity, player_opacity);
                            if let Some(background) = render_cell.style.background.as_mut() {
                                *background = mist.alpha_composite(*background);
                            }
                            fb.set_cell_relative_to_ctx(ctx, coord, depth, render_cell);
                        }
                    });
                }
            }
        }
        tiles
    }

    fn render_hints(&self, ctx: Ctx, fb: &mut FrameBuffer, tiles: &HashSet<Tile>) {
        use text::*;
        let stats = self.game.inner_ref().stats();
        let mut hints = Vec::new();
        if self.game.inner_ref().is_player_on_boat() {
            if self.game.inner_ref().is_driving() {
                hints.push(StyledString {
                    string: format!("Press `e' to stop driving the boat.\n\n"),
                    style: Style::plain_text(),
                });
            } else {
                hints.push(StyledString {
                    string: format!("Press `e' standing on ░ to drive the boat.\n\n"),
                    style: Style::plain_text(),
                });
            }
        }
        if tiles.contains(&Tile::Junk) {
            hints.push(StyledString {
                string: format!("Walk over junk (*) to pick it up.\n\n"),
                style: Style::plain_text(),
            });
        }
        if tiles.contains(&Tile::UnimportantNpc) {
            hints.push(StyledString {
                string: format!("Walk into friendly characters (&) to converse.\n\n"),
                style: Style::plain_text(),
            });
        }
        if tiles.contains(&Tile::Shop) {
            hints.push(StyledString {
                string: format!("Walk into innkeeper ($) to converse.\n\n"),
                style: Style::plain_text(),
            });
        }
        if tiles.contains(&Tile::Button) {
            hints.push(StyledString {
                string: format!("Walk into the gate lever (/) to open the gate.\n\n"),
                style: Style::plain_text(),
            });
        }
        if stats.day.current() < 100
            && stats.day.current() > 0
            && !self.game.inner_ref().is_player_inside()
        {
            hints.push(StyledString {
                    string: format!("\"We'd better get inside, 'cause it'll be dark soon, \nand they mostly come at night...mostly\"\n\n"),
                    style: Style::plain_text(),
                });
        }
        if stats.fuel.current() < 50 {
            hints.push(StyledString {
                string: format!("You are almost out of fuel!\n\n"),
                style: Style::plain_text(),
            });
        }
        if stats.health.current() == 1 {
            hints.push(StyledString {
                string: format!("You are barely clinging to consciousness...\n\n"),
                style: Style::plain_text(),
            });
        }
        if tiles.contains(&Tile::Beast) {
            hints.push(StyledString {
                string: format!("Beasts (b) move towards you on their turn.\n\n"),
                style: Style::plain_text(),
            });
        }
        if tiles.contains(&Tile::Ghost) {
            hints.push(StyledString {
                string: format!("Ghosts (g) can move diagonally.\n\n"),
                style: Style::plain_text(),
            });
        }
        if self.game.inner_ref().is_player_outside_at_night() {
            hints.push(StyledString {
                string: format!("GET INSIDE\n\n"),
                style: Style::plain_text().with_bold(true),
            });
        }
        Text::new(hints).render(&(), ctx, fb);
    }

    fn render_ui(&self, ctx: Ctx, fb: &mut FrameBuffer) {
        use text::*;
        if !self.game.inner_ref().has_been_on_boat() {
            return;
        }
        let stats = self.game.inner_ref().stats();
        let activity = if self.game.inner_ref().is_driving() {
            "Driving Boat   "
        } else {
            "On Foot        "
        };
        let activity_text = StyledString {
            string: activity.to_string(),
            style: Style::plain_text().with_bold(true),
        };
        let day_text = StyledString {
            string: format!("Day {}  ", self.game.inner_ref().current_day()),
            style: Style::plain_text().with_bold(true),
        };
        fn meter_text(name: &str, meter: &Meter) -> Vec<StyledString> {
            vec![
                StyledString {
                    string: format!("{}: ", name),
                    style: Style::plain_text().with_bold(true),
                },
                StyledString {
                    string: format!("{}/{}  ", meter.current(), meter.max()),
                    style: Style::plain_text().with_bold(true),
                },
            ]
        }
        let text = vec![
            vec![activity_text, day_text],
            meter_text("Health", &stats.health),
            meter_text("Fuel", &stats.fuel),
            meter_text("Light", &stats.day),
            meter_text("Junk", &stats.junk),
        ];
        Text::new(text.concat()).render(&(), ctx, fb);
    }

    fn render_messages(&self, ctx: Ctx, fb: &mut FrameBuffer) {
        use text::*;
        let max = 4;
        let mut messages: Vec<(usize, String)> = Vec::new();
        for m in self.game.inner_ref().messages().iter().rev() {
            if messages.len() >= max {
                break;
            }
            if let Some((ref mut count, last)) = messages.last_mut() {
                if last == m {
                    *count += 1;
                    continue;
                }
            }
            messages.push((1, m.clone()));
        }
        for (i, (count, m)) in messages.into_iter().enumerate() {
            let string = if count == 1 {
                m
            } else {
                format!("{} (x{})", m, count)
            };
            let alpha = 255 - (i as u8 * 50);
            let styled_string = StyledString {
                string,
                style: Style::plain_text().with_foreground(Rgba32::new_grey(255).with_a(alpha)),
            };
            let offset = max as i32 - i as i32 - 1;
            styled_string.render(&(), ctx.add_y(offset), fb);
        }
    }

    pub fn render_side_ui(&self, ctx: Ctx, fb: &mut FrameBuffer) {
        use text::*;
        let game = self.game.inner_ref();
        if !game.has_talked_to_npc() {
            return;
        }
        let mut text_parts = vec![StyledString {
            string: format!("Passengers:\n\n"),
            style: Style::plain_text(),
        }];
        let passengers = game.passengers();
        for (i, &npc) in passengers.iter().enumerate() {
            let i = i + 1;
            let name = npc.name();
            let ability_name = npc.ability_name();
            let usage = game.npc_action(npc).unwrap();
            text_parts.push(StyledString {
                string: format!("{i}. {name}\n"),
                style: Style::plain_text(),
            });
            text_parts.push(StyledString {
                string: format!("   {ability_name} {}/{}\n\n", usage.current(), usage.max()),
                style: Style::plain_text().with_bold(true),
            });
        }
        for i in passengers.len()..(self.game.inner_ref().num_seats() as usize) {
            let i = i + 1;
            text_parts.push(StyledString {
                string: format!("{i}. (empty)\n\n\n"),
                style: Style::plain_text(),
            });
        }
        Text::new(text_parts).render(&(), ctx, fb);
    }

    pub fn render(&self, ctx: Ctx, fb: &mut FrameBuffer) {
        let tiles = self.render_game(ctx, fb);
        self.render_hints(ctx.add_xy(1, 1).add_depth(20), fb, &tiles);
        self.render_messages(
            ctx.add_xy(1, ctx.bounding_box.size().height() as i32 - 7)
                .add_depth(20),
            fb,
        );
        self.render_ui(
            ctx.add_xy(1, ctx.bounding_box.size().height() as i32 - 2)
                .add_depth(20),
            fb,
        );
        self.render_side_ui(
            ctx.add_xy(ctx.bounding_box.size().width() as i32 - 16, 1)
                .add_depth(20),
            fb,
        );
    }
}

#[derive(Serialize, Deserialize)]
pub struct GameInstanceStorable {
    running_game: RunningGame,
    mist: Mist,
    fade_state: FadeState,
}

impl GameInstanceStorable {
    pub fn into_game_instance(self) -> (GameInstance, witness::Running) {
        let Self {
            running_game,
            mist,
            fade_state,
        } = self;
        let (game, running) = running_game.into_game();
        (
            GameInstance {
                game,
                mist,
                fade_state,
            },
            running,
        )
    }
}
