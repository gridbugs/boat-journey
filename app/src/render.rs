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
        match game_to_render.status {
            GameStatus::Playing => {
                let mut entity_under_cursor = None;
                let mut star_rng = XorShiftRng::seed_from_u64(game_to_render.game.star_rng_seed());
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
                        .view(
                            &buf,
                            context.add_offset(Coord::new(0, MAP_SIZE.height() as i32 * 3)),
                            frame,
                        );
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
                            context.add_offset(Coord::new(0, MAP_SIZE.height() as i32 * 3)),
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
                                context.add_offset(Coord::new(0, MAP_SIZE.height() as i32 * 3)),
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
                                context.add_offset(Coord::new(0, MAP_SIZE.height() as i32 * 3)),
                                frame,
                            );
                        }
                    }
                }
            }
            GameStatus::Over => {
                for entity in game_to_render.game.to_render_entities() {
                    render_entity_game_over(&entity, game_to_render.game, context, frame);
                }
                StringView::new(Style::new().with_foreground(Rgb24::new(255, 0, 0)), wrap::Word::new()).view(
                    "You failed. The slimes overrun the city and CONSUME WHAT REMAINS OF HUMANITY. Press a key to continue...",
                    context.add_offset(Coord::new(0, MAP_SIZE.height() as i32 * 3)),
                    frame,
                );
            }
        }
        if let Some(action_error) = game_to_render.action_error {
            let s = action_error_str(action_error);
            StringView::new(
                Style::new().with_foreground(Rgb24::new(255, 255, 255)),
                wrap::Word::new(),
            )
            .view(
                s,
                context.add_offset(Coord::new(0, MAP_SIZE.height() as i32 * 3 + 1)),
                frame,
            );
        }
        let ui = ui::Ui {
            player: game_to_render.game.player(),
        };
        ui::UiView.view(ui, context.add_offset(Coord::new(39, 0)), frame);
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

mod quad {
    use super::Coord;
    pub const OFFSETS: [Coord; 4] = [
        Coord::new(0, 0),
        Coord::new(1, 0),
        Coord::new(0, 1),
        Coord::new(1, 1),
    ];
}

struct Quad {
    cells: [ViewCell; 4],
}

fn apply_lighting(cell_colour: Rgb24, light_colour: Rgb24) -> Rgb24 {
    let base_colour = cell_colour
        .saturating_add(light_colour.scalar_div(4))
        .saturating_sub(light_colour.complement().scalar_div(1));
    base_colour.normalised_mul(light_colour)
}

impl Quad {
    fn new_repeating(to_repeat: ViewCell) -> Self {
        Self {
            cells: [to_repeat, to_repeat, to_repeat, to_repeat],
        }
    }
    fn enumerate<'a>(&'a self) -> impl 'a + Iterator<Item = (Coord, ViewCell)> {
        quad::OFFSETS
            .iter()
            .cloned()
            .zip(self.cells.iter().cloned())
    }
    fn new_wall_front(front_col: Rgb24, top_col: Rgb24) -> Self {
        let front = ViewCell::new()
            .with_character(' ')
            .with_background(front_col);
        let top = ViewCell::new()
            .with_character('█')
            .with_background(front_col)
            .with_foreground(top_col);
        Self {
            cells: [top, top, front, front],
        }
    }
    fn new_wall_top(top: Rgb24) -> Self {
        let top = ViewCell::new().with_character(' ').with_background(top);
        Self::new_repeating(top)
    }
    fn new_floor(foreground: Rgb24, background: Rgb24) -> Self {
        let base = ViewCell::new()
            .with_foreground(foreground)
            .with_background(background);
        Self {
            cells: [
                base.with_character('▗'),
                base.with_character('▖'),
                base.with_character('▝'),
                base.with_character('▘'),
            ],
        }
    }
    fn new_door_closed(foreground: Rgb24, background: Rgb24) -> Self {
        let base = ViewCell::new()
            .with_foreground(background)
            .with_background(foreground);
        Self {
            cells: [
                base.with_character('▘'),
                base.with_character('▝'),
                base.with_character('▖'),
                base.with_character('▗'),
            ],
        }
    }
    fn new_door_open(foreground: Rgb24, background: Rgb24) -> Self {
        let base = ViewCell::new()
            .with_foreground(foreground)
            .with_background(background);
        Self {
            cells: [
                base.with_character('▄'),
                base.with_character('▄'),
                base.with_character('▀'),
                base.with_character('▀'),
            ],
        }
    }
    fn new_stairs(foreground: Rgb24, background: Rgb24) -> Self {
        let base = ViewCell::new().with_bold(true);
        Self {
            cells: [
                base.with_character('▝')
                    .with_foreground(background)
                    .with_background(foreground),
                base.with_character(' ').with_background(background),
                base.with_character(' ').with_background(foreground),
                base.with_character('▝')
                    .with_foreground(background)
                    .with_background(foreground),
            ],
        }
    }
    fn new_player(foreground: Rgb24) -> Self {
        let base = ViewCell::new().with_bold(true).with_foreground(foreground);
        Self {
            cells: [
                base.with_character('╔'),
                base.with_character('╗'),
                base.with_character('╚'),
                base.with_character('╩'),
            ],
        }
    }
    fn new_slime(
        character: char,
        foreground: Rgb24,
        background: Rgb24,
        hit_points: u32,
        next_action: NpcAction,
    ) -> Self {
        let base = ViewCell::new()
            .with_background(background)
            .with_foreground(foreground);
        let action_character = match next_action {
            NpcAction::Wait => ' ',
            NpcAction::Walk(direction) => match direction {
                CardinalDirection::North => '↑',
                CardinalDirection::East => '→',
                CardinalDirection::South => '↓',
                CardinalDirection::West => '←',
            },
        };
        Self {
            cells: [
                base.with_character(character)
                    .with_bold(true)
                    .with_foreground(foreground),
                base.with_character(action_character),
                base.with_character(std::char::from_digit((hit_points / 10) % 10, 10).unwrap()),
                base.with_character(std::char::from_digit(hit_points % 10, 10).unwrap()),
            ],
        }
    }
    fn new_attack(foreground: Rgb24, special: bool) -> Self {
        let base = ViewCell::new().with_foreground(foreground).with_bold(true);
        Self {
            cells: [
                base.with_character('A'),
                base.with_character('t'),
                base.with_character('k'),
                base.with_character(if special { '*' } else { ' ' }),
            ],
        }
    }
    fn new_defend(foreground: Rgb24, special: bool) -> Self {
        let base = ViewCell::new().with_foreground(foreground).with_bold(true);
        Self {
            cells: [
                base.with_character('D'),
                base.with_character('e'),
                base.with_character('f'),
                base.with_character(if special { '*' } else { ' ' }),
            ],
        }
    }
    fn new_tech(foreground: Rgb24, special: bool) -> Self {
        let base = ViewCell::new().with_foreground(foreground).with_bold(true);
        Self {
            cells: [
                base.with_character('T'),
                base.with_character('c'),
                base.with_character('h'),
                base.with_character(if special { '*' } else { ' ' }),
            ],
        }
    }
    fn apply_lighting(&mut self, light_colour: Rgb24) {
        for view_cell in self.cells.iter_mut() {
            if let Some(foreground) = view_cell.style.foreground.as_mut() {
                *foreground = apply_lighting(*foreground, light_colour);
            }
            if let Some(background) = view_cell.style.background.as_mut() {
                *background = apply_lighting(*background, light_colour);
            }
        }
    }
}

fn entity_to_quad_visible(entity: &ToRenderEntity, game: &Game, game_over: bool) -> Quad {
    match entity.tile {
        Tile::Player => Quad::new_player(Rgb24::new(255, 255, 255)),
        Tile::Window(_) | Tile::Floor => {
            Quad::new_floor(Rgb24::new(0, 187, 187), Rgb24::new(0, 127, 127))
        }
        Tile::Wall => {
            let below = entity.coord + Coord::new(0, 1);
            if game.contains_wall(below)
                && (game_over || !game.visibility_grid().is_coord_never_visible(below))
            {
                Quad::new_wall_top(Rgb24::new(255, 0, 255))
            } else {
                Quad::new_wall_front(Rgb24::new(127, 0, 127), Rgb24::new(255, 0, 255))
            }
        }
        Tile::DoorClosed(_) => {
            Quad::new_door_closed(Rgb24::new(255, 127, 255), Rgb24::new(127, 0, 127))
        }
        Tile::DoorOpen(_) => {
            Quad::new_door_open(Rgb24::new(255, 127, 255), Rgb24::new(0, 127, 127))
        }
        Tile::Stairs => Quad::new_stairs(Rgb24::new(255, 255, 255), Rgb24::new(0, 127, 127)),
    }
}

fn entity_to_quad_remembered(entity: &ToRenderEntity, game: &Game) -> Option<Quad> {
    let foreground = Rgb24::new_grey(63);
    let background = Rgb24::new_grey(15);
    let quad = match entity.tile {
        Tile::Window(_) | Tile::Floor => Quad::new_floor(foreground, background),
        Tile::Wall => {
            if game.contains_wall(entity.coord + Coord::new(0, 1)) {
                Quad::new_wall_top(foreground)
            } else {
                Quad::new_wall_front(background, foreground)
            }
        }
        Tile::DoorClosed(_) => Quad::new_door_closed(foreground, background),
        Tile::DoorOpen(_) => Quad::new_door_closed(foreground, background),
        Tile::Stairs => Quad::new_stairs(foreground, background),
        Tile::Player => Quad::new_player(foreground),
    };
    Some(quad)
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

fn render_quad<F: Frame, C: ColModify>(
    coord: Coord,
    depth: i8,
    quad: &Quad,
    context: ViewContext<C>,
    frame: &mut F,
) {
    for (offset, view_cell) in quad.enumerate() {
        let output_coord = coord * 3 + offset;
        frame.set_cell_relative(output_coord, depth, view_cell, context);
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

fn render_entity_game_over<F: Frame, C: ColModify>(
    entity: &ToRenderEntity,
    game: &Game,
    context: ViewContext<C>,
    frame: &mut F,
) {
    let mut quad = entity_to_quad_visible(entity, game, true);
    let depth = layer_depth(entity.layer);
    quad.apply_lighting(Rgb24::new(255, 87, 31));
    render_quad(entity.coord, depth, &quad, context, frame);
}

fn tile_str(tile: Tile) -> Option<&'static str> {
    match tile {
        Tile::Player => Some("yourself"),
        Tile::DoorClosed(_) | Tile::DoorOpen(_) => Some("a door"),
        Tile::Wall => Some("a wall"),
        Tile::Floor => Some("the floor"),
        Tile::Window(_) => Some("a window"),
        Tile::Stairs => Some("a staircase leading further down"),
    }
}

fn action_error_str(action_error: ActionError) -> &'static str {
    match action_error {
        ActionError::WalkIntoSolidCell => "You can't walk there",
    }
}
