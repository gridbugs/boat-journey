use crate::{blink::Blink, colours, depth, game::GameStatus, tile_3x3, ui};
use chargrid::render::{
    blend_mode, ColModify, Coord, Frame, Rgb24, Size, Style, View, ViewCell, ViewContext,
};
use chargrid::text::{wrap, StringView, StringViewSingleLine, TextView};
use direction::CardinalDirection;
use line_2d::{Config as LineConfig, LineSegment};
use orbital_decay_game::{
    player::RangedWeaponSlot, ActionError, CellVisibility, Game, Layer, NpcAction, Tile,
    ToRenderEntity, VisibilityGrid, WarningLight, MAP_SIZE,
};
use rand::{Rng, SeedableRng};
use rand_xorshift::XorShiftRng;
use std::time::Duration;

#[derive(Clone, Copy)]
pub enum Mode {
    Normal,
    Aim { slot: RangedWeaponSlot },
    Examine { target: Coord },
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
                let vis_count = game_to_render.game.visibility_grid().count();
                for (coord, visibility_cell) in game_to_render.game.visibility_grid().enumerate() {
                    match visibility_cell.visibility(vis_count) {
                        CellVisibility::CurrentlyVisibleWithLightColour(Some(light_colour)) => {
                            let light_colour = match game_to_render.game.warning_light(coord) {
                                Some(WarningLight::NoAir) => {
                                    Rgb24::new(127, 127, 255).normalised_mul(light_colour)
                                }
                                Some(WarningLight::Decompression) => {
                                    Rgb24::new(255, 127, 127).normalised_mul(light_colour)
                                }
                                None => light_colour,
                            };
                            tile_3x3::render_3x3_from_visibility(
                                coord,
                                visibility_cell,
                                game_to_render.game,
                                context.compose_col_modify(ColModifyLightBlend { light_colour }),
                                frame,
                            );
                        }
                        CellVisibility::PreviouslyVisible => {
                            tile_3x3::render_3x3_from_visibility_remembered(
                                coord,
                                visibility_cell,
                                game_to_render.game,
                                context.compose_col_modify(ColModifyRemembered),
                                frame,
                            );
                        }
                        CellVisibility::NeverVisible
                        | CellVisibility::CurrentlyVisibleWithLightColour(None) => (),
                    }
                }
                if let Some(mouse_coord) = game_to_render.mouse_coord {
                    let game_coord = mouse_coord / 3;
                    if let Some(visibility_cell_under_cursor) =
                        game_to_render.game.visibility_grid().get_cell(game_coord)
                    {
                        let verb = match visibility_cell_under_cursor.visibility(vis_count) {
                            CellVisibility::CurrentlyVisibleWithLightColour(Some(_)) => {
                                Some(MessageVerb::See)
                            }
                            CellVisibility::PreviouslyVisible => Some(MessageVerb::Remember),
                            CellVisibility::NeverVisible
                            | CellVisibility::CurrentlyVisibleWithLightColour(None) => None,
                        };
                        if let Some(verb) = verb {
                            if let Some(floor) = visibility_cell_under_cursor.tile_layers().floor {
                                entity_under_cursor = Some((floor.tile, verb));
                            }
                            if let Some(feature) =
                                visibility_cell_under_cursor.tile_layers().feature
                            {
                                entity_under_cursor = Some((feature.tile, verb));
                            }
                            if let Some(character) =
                                visibility_cell_under_cursor.tile_layers().character
                            {
                                entity_under_cursor = Some((character.tile, verb));
                            }
                            if let Some(item) = visibility_cell_under_cursor.tile_layers().item {
                                entity_under_cursor = Some((item.tile, verb));
                            }
                        }
                    }
                }
                for entity in game_to_render.game.to_render_entities_realtime() {
                    match game_to_render
                        .game
                        .visibility_grid()
                        .cell_visibility(entity.coord)
                    {
                        CellVisibility::CurrentlyVisibleWithLightColour(Some(light_colour)) => {
                            let context =
                                context.compose_col_modify(ColModifyLightBlend { light_colour });
                            if let Some(tile) = entity.tile {
                                tile_3x3::render_3x3_tile(entity.coord, tile, context, frame);
                            }
                            if entity.particle {
                                if let Some(fade) = entity.fade {
                                    let base_coord = entity.coord * 3;
                                    for offset in Size::new_u16(3, 3).coord_iter_row_major() {
                                        frame.blend_cell_background_relative(
                                            base_coord + offset,
                                            1,
                                            Rgb24::new_grey(187).normalised_mul(light_colour),
                                            (255 - fade) / 10,
                                            blend_mode::LinearInterpolate,
                                            context,
                                        )
                                    }
                                }
                            }
                        }
                        _ => (),
                    }
                }
                if let Some((tile, verb)) = entity_under_cursor {
                    match tile_str(tile) {
                        Some(TileLabel::Name(name)) => {
                            let verb_str = match verb {
                                MessageVerb::Remember => "remember seeing",
                                MessageVerb::See => "see",
                            };
                            let mut buf = String::new();
                            use std::fmt::Write;
                            write!(&mut buf, "You {} {} here.", verb_str, name).unwrap();
                            StringViewSingleLine::new(
                                Style::new().with_foreground(Rgb24::new_grey(255)),
                            )
                            .view(&buf, context, frame);
                        }
                        Some(TileLabel::Literal(literal)) => {
                            let mut buf = String::new();
                            use std::fmt::Write;
                            write!(&mut buf, "{}", literal).unwrap();
                            TextView::new(
                                Style::new().with_foreground(Rgb24::new_grey(255)),
                                wrap::Word::new(),
                            )
                            .view(vec![buf], context, frame);
                        }
                        None => (),
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
                                "Gotta get to the fuel bay on the {}th floor...",
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
                                "FINAL FLOOR. Get to the fuel bay!".to_string(),
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
            GameStatus::Dead => {
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
                StringView::new(
                    Style::new().with_foreground(Rgb24::new(255, 0, 0)),
                    wrap::Word::new(),
                )
                .view("You are dead! Press any key...", context, frame);
            }
            GameStatus::Adrift => {
                render_stars_all(
                    &mut star_rng,
                    context.compose_col_modify(ColModifyAdrift),
                    frame,
                );
                for entity in game_to_render.game.to_render_entities() {
                    let depth = layer_depth(entity.layer);
                    tile_3x3::render_3x3(
                        &entity,
                        game_to_render.game,
                        context.add_depth(depth).compose_col_modify(ColModifyAdrift),
                        frame,
                    );
                }
                StringView::new(
                    Style::new().with_foreground(Rgb24::new(255, 0, 0)),
                    wrap::Word::new(),
                )
                .view(
                    "You drift in space forever! Press any key...",
                    context,
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
            .view(s, context.add_offset(Coord::new(0, 1)), frame);
        }

        let ui = ui::Ui {
            player: game_to_render.game.player(),
            player_info: game_to_render.game.player_info(),
        };
        ui::UiView.view(ui, context.add_offset(Coord::new(64, 4)), frame);
        match game_to_render.mode {
            Mode::Normal => (),
            Mode::Aim { slot } => {
                let slot_str = match slot {
                    RangedWeaponSlot::Slot1 => "Weapon 1",
                    RangedWeaponSlot::Slot2 => "Weapon 2",
                    RangedWeaponSlot::Slot3 => "Weapon 3",
                };
                StringViewSingleLine::new(
                    Style::new()
                        .with_foreground(Rgb24::new(255, 0, 0))
                        .with_bold(true),
                )
                .view(
                    format!("Fire {} in which direction? (escape to cancel)", slot_str).as_str(),
                    context.add_offset(Coord::new(0, 1)),
                    frame,
                );
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
                    context.add_offset(Coord::new(0, 2)),
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
            Layer::Item => 2,
            Layer::Character => 3,
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
        let bg = colours::SPACE_BACKGROUND.saturating_scalar_mul_div(30 + coord.y as u32, 90);
        let (ch, style) = match star {
            Star::None => (' ', Style::new().with_background(bg)),
            Star::Dim => (
                '.',
                Style::new()
                    .with_bold(false)
                    .with_foreground(colours::SPACE_FOREGROUND_DIM)
                    .with_background(bg),
            ),
            Star::Bright => (
                '.',
                Style::new()
                    .with_bold(true)
                    .with_foreground(colours::SPACE_FOREGROUND)
                    .with_background(bg),
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
        let bg = colours::SPACE_BACKGROUND.saturating_scalar_mul_div(30 + coord.y as u32, 90);
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
                    Star::None => (' ', Style::new().with_background(bg)),
                    Star::Dim => (
                        '.',
                        Style::new()
                            .with_bold(false)
                            .with_foreground(colours::SPACE_FOREGROUND_DIM)
                            .with_background(bg),
                    ),
                    Star::Bright => (
                        '.',
                        Style::new()
                            .with_bold(true)
                            .with_foreground(colours::SPACE_FOREGROUND)
                            .with_background(bg),
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
        colour
            .normalised_mul(self.light_colour)
            .saturating_add(self.light_colour.saturating_scalar_mul_div(1, 10))
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

#[derive(Clone, Copy)]
struct ColModifyAdrift;
impl ColModifyAdrift {
    fn apply_lighting(&self, colour: Rgb24) -> Rgb24 {
        let mean = colour.weighted_mean_u16(rgb24::WeightsU16::new(1, 1, 1));
        Rgb24::new(0, 0, mean).saturating_scalar_mul_div(3, 2)
    }
}

impl ColModify for ColModifyAdrift {
    fn foreground(&self, rgb24: Option<Rgb24>) -> Option<Rgb24> {
        rgb24.map(|rgb24| self.apply_lighting(rgb24))
    }
    fn background(&self, rgb24: Option<Rgb24>) -> Option<Rgb24> {
        rgb24.map(|rgb24| self.apply_lighting(rgb24))
    }
}

enum TileLabel {
    Literal(&'static str),
    Name(&'static str),
}

fn tile_str(tile: Tile) -> Option<TileLabel> {
    let label = match tile {
        Tile::Player => TileLabel::Name("yourself"),
        Tile::DoorClosed(_) | Tile::DoorOpen(_) => TileLabel::Name("a door"),
        Tile::Wall | Tile::WallText0 | Tile::WallText1 | Tile::WallText2 | Tile::WallText3 => {
            TileLabel::Name("a wall")
        }
        Tile::Floor | Tile::FuelText0 | Tile::FuelText1 => TileLabel::Name("the floor"),
        Tile::FuelHatch => TileLabel::Name("the fuel bay"),
        Tile::Window(_) => TileLabel::Name("a window"),
        Tile::Stairs => TileLabel::Name("a staircase leading further down"),
        Tile::Zombie => TileLabel::Name("a zombie"),
        Tile::Skeleton => TileLabel::Name("a skeleton"),
        Tile::SkeletonRespawn => TileLabel::Name("a twitching pile of bones"),
        Tile::Boomer => TileLabel::Name("a boomer"),
        Tile::Tank => TileLabel::Name("a tank"),
        Tile::Bullet => return None,
        Tile::Credit1 => TileLabel::Name("a $1 credit chip"),
        Tile::Credit2 => TileLabel::Name("a $2 credit chip"),
        Tile::Upgrade => TileLabel::Name("an upgrade store"),
        Tile::Medkit => TileLabel::Name("a medkit"),
        Tile::Chainsaw => {
            TileLabel::Literal("A chainsaw - melee weapon with high DMG and limited uses.")
        }
        Tile::Shotgun => TileLabel::Literal("A shotgun - high DMG, low PEN."),
        Tile::Railgun => TileLabel::Literal("A railgun - it can shoot through almost anything!"),
        Tile::Rifle => TileLabel::Literal("A rifle - general all-rounder. Boring."),
        Tile::GausCannon => TileLabel::Literal(
            "A gaus cannon - cooks organic matter leaving the hull intact. Ammo is scarce!",
        ),
        Tile::Oxidiser => {
            TileLabel::Literal("An oxidiser - converts organic matter into oxygen.")
        }
        Tile::LifeStealer => {
            TileLabel::Literal("A life stealer - converts the recently deceased into health like some kind of creepy vampire. And you thought the zombies were gross!")
        }
    };
    Some(label)
}

fn action_error_str(action_error: ActionError) -> &'static str {
    match action_error {
        ActionError::WalkIntoSolidCell => "You can't walk there!",
        ActionError::CannotAffordUpgrade => "You can't afford tha!t",
    }
}
