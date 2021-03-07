use crate::{blink::Blink, colours, depth, game::GameStatus, tile_3x3, ui};
use chargrid::render::{
    blend_mode, ColModify, Coord, Frame, Rgb24, Size, Style, View, ViewCell, ViewContext,
};
use chargrid::text::{wrap, StringView, StringViewSingleLine};
use direction::CardinalDirection;
use line_2d::{Config as LineConfig, LineSegment};
use orbital_decay_game::{
    ActionError, CellVisibility, Game, Layer, NpcAction, Tile, ToRenderEntity, VisibilityGrid,
    MAP_SIZE,
};
use rand::{Rng, SeedableRng};
use rand_xorshift::XorShiftRng;
use std::time::Duration;

#[derive(Clone, Copy)]
pub enum Mode {
    Normal,
    Aim {
        blink_duration: Duration,
        target: Coord,
    },
    Examine {
        target: Coord,
    },
}

pub struct GameToRender<'a> {
    pub game: &'a Game,
    pub status: GameStatus,
    pub mouse_coord: Option<Coord>,
    pub mode: Mode,
    pub action_error: Option<ActionError>,
}

pub struct GameView {
    last_offset: Coord,
    blink: Blink,
}

#[derive(Clone, Copy, Debug)]
enum MessageVerb {
    See,
    Remember,
}

impl GameView {
    pub fn new() -> Self {
        Self {
            last_offset: Coord::new(0, 0),
            blink: Blink::new(),
        }
    }

    pub fn absolute_coord_to_game_relative_screen_coord(&self, coord: Coord) -> Coord {
        coord - self.last_offset
    }

    pub fn view<F: Frame, C: ColModify>(
        &mut self,
        game_to_render: GameToRender,
        context: ViewContext<C>,
        frame: &mut F,
    ) {
        let mut star_rng = XorShiftRng::seed_from_u64(game_to_render.game.star_rng_seed());
        match game_to_render.status {
            GameStatus::Playing => {
                let mut entity_under_cursor = None;
                render_stars(
                    game_to_render.game.visibility_grid(),
                    &mut star_rng,
                    context,
                    frame,
                );
                for entity in game_to_render.game.to_render_entities() {
                    render_entity(&entity, game_to_render.game, context, frame);
                    if let Some(mouse_coord) = game_to_render.mouse_coord {
                        let game_coord = mouse_coord / 3;
                        if entity.coord == game_coord {
                            let verb = match game_to_render
                                .game
                                .visibility_grid()
                                .cell_visibility(entity.coord)
                            {
                                CellVisibility::CurrentlyVisibleWithLightColour(Some(_)) => {
                                    Some(MessageVerb::See)
                                }
                                CellVisibility::PreviouslyVisible => Some(MessageVerb::Remember),
                                CellVisibility::NeverVisible
                                | CellVisibility::CurrentlyVisibleWithLightColour(None) => None,
                            };
                            if let Some(verb) = verb {
                                if let Some((max_depth, _tile, _verb)) = entity_under_cursor {
                                    let depth = layer_depth(entity.layer);
                                    if depth > max_depth {
                                        entity_under_cursor = Some((depth, entity.tile, verb));
                                    }
                                } else {
                                    entity_under_cursor =
                                        Some((layer_depth(entity.layer), entity.tile, verb));
                                }
                            }
                        }
                    }
                }
                if let Some((_depth, tile, verb)) = entity_under_cursor {
                    if let Some(description) = tile_str(tile) {
                        let verb_str = match verb {
                            MessageVerb::Remember => "remember seeing",
                            MessageVerb::See => "see",
                        };
                        let mut buf = String::new();
                        use std::fmt::Write;
                        write!(&mut buf, "You {} {} here.", verb_str, description).unwrap();
                        StringViewSingleLine::new(
                            Style::new().with_foreground(Rgb24::new_grey(255)),
                        )
                        .view(&buf, context, frame);
                    }
                } else {
                    let current_level = game_to_render.game.current_level();
                    if current_level == 0 {
                        StringView::new(
                            Style::new()
                                .with_foreground(Rgb24::new_grey(255))
                                .with_bold(true),
                            wrap::Word::new(),
                        )
                        .view(
                            format!(
                                "COMMANDER: The source of the slime is on the {}th floor.",
                                orbital_decay_game::FINAL_LEVEL
                            ),
                            context,
                            frame,
                        );
                    } else {
                        if current_level == orbital_decay_game::FINAL_LEVEL {
                            StringView::new(
                                Style::new()
                                    .with_foreground(Rgb24::new_grey(255))
                                    .with_bold(true),
                                wrap::Word::new(),
                            )
                            .view(
                                "FINAL FLOOR".to_string(),
                                context,
                                frame,
                            );
                        } else {
                            StringViewSingleLine::new(
                                Style::new().with_foreground(Rgb24::new_grey(255)),
                            )
                            .view(
                                format!(
                                    "Floor {}/{}",
                                    current_level,
                                    orbital_decay_game::FINAL_LEVEL
                                ),
                                context,
                                frame,
                            );
                        }
                    }
                }
            }
            GameStatus::Over => {
                render_stars_all(
                    &mut star_rng,
                    context.compose_col_modify(ColModifyDead),
                    frame,
                );
                for entity in game_to_render.game.to_render_entities() {
                    let depth = layer_depth(entity.layer);
                    tile_3x3::render_3x3(
                        &entity,
                        game_to_render.game,
                        context.add_depth(depth).compose_col_modify(ColModifyDead),
                        frame,
                    );
                }
                StringView::new(Style::new().with_foreground(Rgb24::new(255, 0, 0)), wrap::Word::new()).view(
                    "You failed. The slimes overrun the city and CONSUME WHAT REMAINS OF HUMANITY. Press a key to continue...",
                    context,
                    frame,
                );
            }
        }
        let ui = ui::Ui {
            player: game_to_render.game.player(),
            player_info: game_to_render.game.player_info(),
        };
        ui::UiView.view(ui, context.add_offset(Coord::new(65, 4)), frame);
        match game_to_render.mode {
            Mode::Normal => (),
            Mode::Aim {
                blink_duration,
                target,
            } => {
                let aim_coord = target / 3;
                let player_coord = game_to_render.game.player_coord();
                if aim_coord != player_coord {
                    for node in
                        LineSegment::new(player_coord, aim_coord).config_node_iter(LineConfig {
                            exclude_start: true,
                            exclude_end: true,
                        })
                    {
                        if !node.coord.is_valid(orbital_decay_game::MAP_SIZE) {
                            break;
                        }
                        for &offset in &tile_3x3::OFFSETS {
                            let output_coord = node.coord * 3 + offset;
                            frame.blend_cell_background_relative(
                                output_coord,
                                depth::GAME_MAX,
                                Rgb24::new(255, 0, 0),
                                127,
                                blend_mode::LinearInterpolate,
                                context,
                            );
                        }
                    }
                }
                if aim_coord.is_valid(orbital_decay_game::MAP_SIZE) {
                    for &offset in &tile_3x3::OFFSETS {
                        let alpha = self.blink.alpha(blink_duration);
                        let output_coord = aim_coord * 3 + offset;
                        frame.blend_cell_background_relative(
                            output_coord,
                            depth::GAME_MAX,
                            Rgb24::new(255, 0, 0),
                            alpha,
                            blend_mode::LinearInterpolate,
                            context,
                        );
                    }
                }
            }
            Mode::Examine { target } => {
                let game_coord = target / 3;
                if game_coord.is_valid(MAP_SIZE) {
                    for &offset in &tile_3x3::OFFSETS {
                        let alpha = 127;
                        let output_coord = game_coord * 3 + offset;
                        frame.blend_cell_background_relative(
                            output_coord,
                            depth::GAME_MAX,
                            Rgb24::new(255, 255, 0),
                            alpha,
                            blend_mode::LinearInterpolate,
                            context,
                        );
                    }
                }
                StringViewSingleLine::new(Style::new().with_foreground(Rgb24::new_grey(127))).view(
                    "Examining (escape to return to game)",
                    context.add_offset(Coord::new(0, MAP_SIZE.height() as i32 * 3 + 1)),
                    frame,
                );
            }
        }
        if let Some(mouse_coord) = game_to_render.mouse_coord {
            let game_coord = mouse_coord / 3;
            if game_coord.is_valid(MAP_SIZE) {
                for &offset in &tile_3x3::OFFSETS {
                    let alpha = 63;
                    let output_coord = game_coord * 3 + offset;
                    frame.blend_cell_background_relative(
                        output_coord,
                        depth::GAME_MAX,
                        Rgb24::new(255, 255, 0),
                        alpha,
                        blend_mode::LinearInterpolate,
                        context,
                    );
                }
            }
        }
    }
}

pub fn layer_depth(layer: Option<Layer>) -> i8 {
    if let Some(layer) = layer {
        match layer {
            Layer::Floor => 0,
            Layer::Feature => 1,
            Layer::Character => 2,
        }
    } else {
        depth::GAME_MAX - 1
    }
}

pub fn render_stars_all<R: Rng, F: Frame, C: ColModify>(
    star_rng: &mut R,
    context: ViewContext<C>,
    frame: &mut F,
) {
    enum Star {
        None,
        Dim,
        Bright,
    }
    for coord in context.size.coord_iter_row_major() {
        let star = if star_rng.gen::<u32>() % 60 == 0 {
            Star::Bright
        } else if star_rng.gen::<u32>() % 60 == 0 {
            Star::Dim
        } else {
            Star::None
        };
        let (ch, style) = match star {
            Star::None => (' ', Style::new().with_background(colours::SPACE_BACKGROUND)),
            Star::Dim => (
                '.',
                Style::new()
                    .with_bold(false)
                    .with_foreground(colours::SPACE_FOREGROUND_DIM)
                    .with_background(colours::SPACE_BACKGROUND),
            ),
            Star::Bright => (
                '.',
                Style::new()
                    .with_bold(true)
                    .with_foreground(colours::SPACE_FOREGROUND)
                    .with_background(colours::SPACE_BACKGROUND),
            ),
        };
        frame.set_cell_relative(
            coord,
            0,
            ViewCell::new().with_character(ch).with_style(style),
            context,
        );
    }
}

pub fn render_stars<R: Rng, F: Frame, C: ColModify>(
    visibility_grid: &VisibilityGrid,
    star_rng: &mut R,
    context: ViewContext<C>,
    frame: &mut F,
) {
    enum Star {
        None,
        Dim,
        Bright,
    }
    for coord in context.size.coord_iter_row_major() {
        let visibility = visibility_grid.cell_visibility(coord / 3);
        let star = if star_rng.gen::<u32>() % 60 == 0 {
            Star::Bright
        } else if star_rng.gen::<u32>() % 60 == 0 {
            Star::Dim
        } else {
            Star::None
        };
        match visibility {
            CellVisibility::NeverVisible => {
                frame.set_cell_relative(
                    coord,
                    0,
                    ViewCell::new()
                        .with_character(' ')
                        .with_background(Rgb24::new_grey(0)),
                    context,
                );
            }
            CellVisibility::PreviouslyVisible => {
                let num = 1;
                let denom = 4;
                let (ch, style) = match star {
                    Star::None => (
                        ' ',
                        Style::new()
                            .with_foreground(
                                colours::SPACE_FOREGROUND_DIM.saturating_scalar_mul_div(num, denom),
                            )
                            .with_background(
                                colours::SPACE_BACKGROUND.saturating_scalar_mul_div(num, denom),
                            ),
                    ),
                    Star::Dim => (
                        '.',
                        Style::new()
                            .with_foreground(
                                colours::SPACE_FOREGROUND_DIM.saturating_scalar_mul_div(num, denom),
                            )
                            .with_background(
                                colours::SPACE_BACKGROUND.saturating_scalar_mul_div(num, denom),
                            ),
                    ),
                    Star::Bright => (
                        '.',
                        Style::new()
                            .with_bold(true)
                            .with_foreground(
                                colours::SPACE_FOREGROUND_DIM.saturating_scalar_mul_div(num, denom),
                            )
                            .with_background(
                                colours::SPACE_BACKGROUND.saturating_scalar_mul_div(num, denom),
                            ),
                    ),
                };
                frame.set_cell_relative(
                    coord,
                    0,
                    ViewCell::new().with_character(ch).with_style(style),
                    context,
                );
            }
            CellVisibility::CurrentlyVisibleWithLightColour(_) => {
                let (ch, style) = match star {
                    Star::None => (' ', Style::new().with_background(colours::SPACE_BACKGROUND)),
                    Star::Dim => (
                        '.',
                        Style::new()
                            .with_bold(false)
                            .with_foreground(colours::SPACE_FOREGROUND_DIM)
                            .with_background(colours::SPACE_BACKGROUND),
                    ),
                    Star::Bright => (
                        '.',
                        Style::new()
                            .with_bold(true)
                            .with_foreground(colours::SPACE_FOREGROUND)
                            .with_background(colours::SPACE_BACKGROUND),
                    ),
                };
                frame.set_cell_relative(
                    coord,
                    0,
                    ViewCell::new().with_character(ch).with_style(style),
                    context,
                );
            }
        }
    }
}

#[derive(Clone, Copy)]
struct ColModifyLightBlend {
    light_colour: Rgb24,
}

impl ColModifyLightBlend {
    fn apply_lighting(&self, colour: Rgb24) -> Rgb24 {
        let base_colour = colour;
        base_colour.normalised_mul(self.light_colour)
    }
}

impl ColModify for ColModifyLightBlend {
    fn foreground(&self, rgb24: Option<Rgb24>) -> Option<Rgb24> {
        rgb24.map(|rgb24| self.apply_lighting(rgb24))
    }
    fn background(&self, rgb24: Option<Rgb24>) -> Option<Rgb24> {
        rgb24.map(|rgb24| self.apply_lighting(rgb24))
    }
}

#[derive(Clone, Copy)]
struct ColModifyRemembered;
impl ColModifyRemembered {
    fn apply_lighting(&self, colour: Rgb24) -> Rgb24 {
        let mean = colour.weighted_mean_u16(rgb24::WeightsU16::new(1, 1, 1));
        Rgb24::new_grey(mean).saturating_scalar_mul_div(1, 2)
    }
}

impl ColModify for ColModifyRemembered {
    fn foreground(&self, rgb24: Option<Rgb24>) -> Option<Rgb24> {
        rgb24.map(|rgb24| self.apply_lighting(rgb24))
    }
    fn background(&self, rgb24: Option<Rgb24>) -> Option<Rgb24> {
        rgb24.map(|rgb24| self.apply_lighting(rgb24))
    }
}

#[derive(Clone, Copy)]
struct ColModifyDead;
impl ColModifyDead {
    fn apply_lighting(&self, colour: Rgb24) -> Rgb24 {
        let mean = colour.weighted_mean_u16(rgb24::WeightsU16::new(1, 1, 1));
        Rgb24::new(mean, 0, 0).saturating_scalar_mul_div(3, 2)
    }
}

impl ColModify for ColModifyDead {
    fn foreground(&self, rgb24: Option<Rgb24>) -> Option<Rgb24> {
        rgb24.map(|rgb24| self.apply_lighting(rgb24))
    }
    fn background(&self, rgb24: Option<Rgb24>) -> Option<Rgb24> {
        rgb24.map(|rgb24| self.apply_lighting(rgb24))
    }
}

pub fn render_entity<F: Frame, C: ColModify>(
    entity: &ToRenderEntity,
    game: &Game,
    context: ViewContext<C>,
    frame: &mut F,
) {
    match game.visibility_grid().cell_visibility(entity.coord) {
        CellVisibility::CurrentlyVisibleWithLightColour(Some(light_colour)) => {
            let context = context.compose_col_modify(ColModifyLightBlend { light_colour });
            let depth = layer_depth(entity.layer);
            tile_3x3::render_3x3(entity, game, context.add_depth(depth), frame);
        }
        CellVisibility::PreviouslyVisible => {
            let context = context.compose_col_modify(ColModifyRemembered);
            let depth = layer_depth(entity.layer);
            tile_3x3::render_3x3(entity, game, context.add_depth(depth), frame);
        }
        CellVisibility::NeverVisible | CellVisibility::CurrentlyVisibleWithLightColour(None) => (),
    }
}

fn tile_str(tile: Tile) -> Option<&'static str> {
    match tile {
        Tile::Player => Some("yourself"),
        Tile::DoorClosed(_) | Tile::DoorOpen(_) => Some("a door"),
        Tile::Wall => Some("a wall"),
        Tile::Floor => Some("the floor"),
        Tile::Window(_) => Some("a window"),
        Tile::Stairs => Some("a staircase leading further down"),
        Tile::Zombie => Some("a zombie"),
    }
}

fn action_error_str(action_error: ActionError) -> &'static str {
    match action_error {
        ActionError::WalkIntoSolidCell => "You can't walk there",
    }
}
