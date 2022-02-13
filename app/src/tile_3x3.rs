use crate::colours;
use chargrid::core::prelude::*;
use grid_2d::coord_2d::{Axis, Coord, Size};
use orbital_decay_game::{EntityTile, Game, Tile, ToRenderEntity, VisibilityCell};

struct StrStyle(Style);
impl StrStyle {
    fn new(style: Style) -> Self {
        Self(style)
    }
}

impl Component for StrStyle {
    type Output = ();
    type State = str;
    fn render(&self, state: &Self::State, ctx: Ctx, fb: &mut FrameBuffer) {
        for (i, ch) in state.chars().enumerate() {
            let offset = Coord { x: i as i32, y: 0 };
            let render_cell = RenderCell {
                character: Some(ch),
                style: self.0,
            };
            fb.set_cell_relative_to_ctx(ctx, offset, 0, render_cell);
        }
    }
    fn update(&mut self, _state: &mut Self::State, _ctx: Ctx, _event: Event) -> Self::Output {}
    fn size(&self, state: &Self::State, _ctx: Ctx) -> Size {
        Size::new_u16(state.len() as u16, 1)
    }
}

pub fn render_3x3_from_visibility(
    coord: Coord,
    visibility_cell: &VisibilityCell,
    game: &Game,
    ctx: Ctx,
    fb: &mut FrameBuffer,
) {
    let ctx = ctx.add_offset(coord * 3);
    let mut render_tile = |entity, tile, ctx| match tile {
        Tile::Wall => {
            let below = coord + Coord::new(0, 1);
            if let Some(render_cell) = game.visibility_grid().get_cell(below) {
                if render_cell.tile_layers().feature.is_some() {
                    wall_top(ctx, fb);
                } else {
                    wall_front(ctx, fb);
                }
            } else {
                wall_front(ctx, fb);
            }
        }
        Tile::WallText0 => wall_front_0(ctx, fb),
        Tile::WallText1 => wall_front_1(ctx, fb),
        Tile::WallText2 => wall_front_2(ctx, fb),
        Tile::WallText3 => wall_front_3(ctx, fb),
        Tile::Floor => floor(ctx, fb),
        Tile::FuelText0 => fuel_text_0(ctx, fb),
        Tile::FuelText1 => fuel_text_1(ctx, fb),
        Tile::FuelHatch => fuel_hatch(ctx, fb),
        Tile::Player => player(ctx, fb),
        Tile::Window(Axis::Y) => {
            let below = coord + Coord::new(0, 1);
            window_y(game.contains_floor(below), ctx, fb);
        }
        Tile::Window(Axis::X) => window_x(ctx, fb),
        Tile::DoorOpen(Axis::X) => door_open_x(ctx, fb),
        Tile::DoorOpen(Axis::Y) => door_open_y(ctx, fb),
        Tile::DoorClosed(Axis::X) => door_closed_x(ctx, fb),
        Tile::DoorClosed(Axis::Y) => door_closed_y(ctx, fb),
        Tile::Stairs => stairs(ctx, fb),
        Tile::Bullet => bullet(ctx, fb),
        Tile::Zombie => {
            if let Some(entity) = game.to_render_entity(entity) {
                zombie(&entity, ctx, fb);
            }
        }
        Tile::Skeleton => {
            if let Some(entity) = game.to_render_entity(entity) {
                skeleton(&entity, ctx, fb);
            }
        }
        Tile::SkeletonRespawn => {
            if let Some(entity) = game.to_render_entity(entity) {
                skeleton_respawn(&entity, ctx, fb);
            }
        }
        Tile::Boomer => {
            if let Some(entity) = game.to_render_entity(entity) {
                boomer(&entity, ctx, fb);
            }
        }
        Tile::Tank => {
            if let Some(entity) = game.to_render_entity(entity) {
                tank(&entity, ctx, fb);
            }
        }
        Tile::Credit1 => credit1(ctx, fb),
        Tile::Credit2 => credit2(ctx, fb),
        Tile::Upgrade => upgrade(ctx, fb),
        Tile::Map => map(ctx, fb),
        Tile::MapLocked => map_locked(ctx, fb),
        Tile::Chainsaw => chainsaw(ctx, fb),
        Tile::Shotgun => shotgun(ctx, fb),
        Tile::Railgun => railgun(ctx, fb),
        Tile::Rifle => rifle(ctx, fb),
        Tile::GausCannon => gaus_cannon(ctx, fb),
        Tile::Oxidiser => oxidiser(ctx, fb),
        Tile::LifeStealer => life_stealer(ctx, fb),
        Tile::Medkit => medkit(ctx, fb),
    };
    let tile_layers = visibility_cell.tile_layers();
    if let Some(EntityTile { entity, tile }) = tile_layers.floor {
        render_tile(entity, tile, ctx.add_depth(0));
    }
    if let Some(EntityTile { entity, tile }) = tile_layers.feature {
        render_tile(entity, tile, ctx.add_depth(1));
    }
    if let Some(EntityTile { entity, tile }) = tile_layers.item {
        render_tile(entity, tile, ctx.add_depth(2));
    }
    if let Some(EntityTile { entity, tile }) = tile_layers.character {
        render_tile(entity, tile, ctx.add_depth(3));
    }
}

pub fn render_3x3_from_visibility_remembered(
    coord: Coord,
    visibility_cell: &VisibilityCell,
    game: &Game,
    ctx: Ctx,
    fb: &mut FrameBuffer,
) {
    let ctx = ctx.add_offset(coord * 3);
    let mut render_tile = |tile, ctx| match tile {
        Tile::Wall => {
            let below = coord + Coord::new(0, 1);
            if let Some(render_cell) = game.visibility_grid().get_cell(below) {
                if render_cell.tile_layers().feature.is_some() {
                    wall_top(ctx, fb);
                } else {
                    wall_front(ctx, fb);
                }
            } else {
                wall_front(ctx, fb);
            }
        }
        Tile::WallText0 => wall_front_0(ctx, fb),
        Tile::WallText1 => wall_front_1(ctx, fb),
        Tile::WallText2 => wall_front_2(ctx, fb),
        Tile::WallText3 => wall_front_3(ctx, fb),
        Tile::Floor => floor(ctx, fb),
        Tile::FuelText0 => fuel_text_0(ctx, fb),
        Tile::FuelText1 => fuel_text_1(ctx, fb),
        Tile::FuelHatch => fuel_hatch(ctx, fb),
        Tile::Player => player(ctx, fb),
        Tile::Window(Axis::Y) => {
            let below = coord + Coord::new(0, 1);
            window_y(game.contains_floor(below), ctx, fb);
        }
        Tile::Window(Axis::X) => window_x(ctx, fb),
        Tile::DoorOpen(Axis::X) => door_open_x(ctx, fb),
        Tile::DoorOpen(Axis::Y) => door_open_y(ctx, fb),
        Tile::DoorClosed(Axis::X) => door_closed_x(ctx, fb),
        Tile::DoorClosed(Axis::Y) => door_closed_y(ctx, fb),
        Tile::Stairs => stairs(ctx, fb),
        Tile::Bullet => bullet(ctx, fb),
        Tile::Zombie => (),
        Tile::Skeleton => (),
        Tile::SkeletonRespawn => (),
        Tile::Tank => (),
        Tile::Boomer => (),
        Tile::Credit1 => credit1(ctx, fb),
        Tile::Credit2 => credit2(ctx, fb),
        Tile::Upgrade => upgrade(ctx, fb),
        Tile::Map => map(ctx, fb),
        Tile::MapLocked => map_locked(ctx, fb),
        Tile::Chainsaw => chainsaw(ctx, fb),
        Tile::Shotgun => shotgun(ctx, fb),
        Tile::Railgun => railgun(ctx, fb),
        Tile::Rifle => rifle(ctx, fb),
        Tile::GausCannon => gaus_cannon(ctx, fb),
        Tile::Oxidiser => oxidiser(ctx, fb),
        Tile::LifeStealer => life_stealer(ctx, fb),
        Tile::Medkit => medkit(ctx, fb),
    };
    let tile_layers = visibility_cell.tile_layers();
    if let Some(EntityTile { entity: _, tile }) = tile_layers.floor {
        render_tile(tile, ctx.add_depth(0));
    }
    if let Some(EntityTile { entity: _, tile }) = tile_layers.feature {
        render_tile(tile, ctx.add_depth(1));
    }
    if let Some(EntityTile { entity: _, tile }) = tile_layers.item {
        render_tile(tile, ctx.add_depth(2));
    }
    if let Some(EntityTile { entity: _, tile }) = tile_layers.character {
        render_tile(tile, ctx.add_depth(3));
    }
}

pub fn render_3x3_tile(coord: Coord, tile: Tile, ctx: Ctx, fb: &mut FrameBuffer) {
    let ctx = ctx.add_offset(coord * 3);
    match tile {
        Tile::Bullet => bullet(ctx, fb),
        _ => (),
    }
}

pub fn render_3x3(entity: &ToRenderEntity, game: &Game, ctx: Ctx, fb: &mut FrameBuffer) {
    let ctx = ctx.add_offset(entity.coord * 3);
    match entity.tile {
        Tile::Wall => {
            let below = entity.coord + Coord::new(0, 1);
            if game.contains_wall_like(below) {
                wall_top(ctx, fb);
            } else {
                wall_front(ctx, fb);
            }
        }
        Tile::WallText0 => wall_front_0(ctx, fb),
        Tile::WallText1 => wall_front_1(ctx, fb),
        Tile::WallText2 => wall_front_2(ctx, fb),
        Tile::WallText3 => wall_front_3(ctx, fb),
        Tile::Floor => floor(ctx, fb),
        Tile::FuelText0 => fuel_text_0(ctx, fb),
        Tile::FuelText1 => fuel_text_1(ctx, fb),
        Tile::FuelHatch => fuel_hatch(ctx, fb),
        Tile::Player => player(ctx, fb),
        Tile::Window(Axis::Y) => {
            let below = entity.coord + Coord::new(0, 1);
            window_y(game.contains_floor(below), ctx, fb);
        }
        Tile::Window(Axis::X) => window_x(ctx, fb),
        Tile::DoorOpen(Axis::X) => door_open_x(ctx, fb),
        Tile::DoorOpen(Axis::Y) => door_open_y(ctx, fb),
        Tile::DoorClosed(Axis::X) => door_closed_x(ctx, fb),
        Tile::DoorClosed(Axis::Y) => door_closed_y(ctx, fb),
        Tile::Stairs => stairs(ctx, fb),
        Tile::Bullet => bullet(ctx, fb),
        Tile::Zombie => zombie(entity, ctx, fb),
        Tile::Skeleton => skeleton(entity, ctx, fb),
        Tile::SkeletonRespawn => skeleton_respawn(entity, ctx, fb),
        Tile::Boomer => boomer(entity, ctx, fb),
        Tile::Tank => tank(entity, ctx, fb),
        Tile::Credit1 => credit1(ctx, fb),
        Tile::Credit2 => credit2(ctx, fb),
        Tile::Upgrade => upgrade(ctx, fb),
        Tile::Map => map(ctx, fb),
        Tile::MapLocked => map_locked(ctx, fb),
        Tile::Chainsaw => chainsaw(ctx, fb),
        Tile::Shotgun => shotgun(ctx, fb),
        Tile::Railgun => railgun(ctx, fb),
        Tile::Rifle => rifle(ctx, fb),
        Tile::GausCannon => gaus_cannon(ctx, fb),
        Tile::Oxidiser => oxidiser(ctx, fb),
        Tile::LifeStealer => life_stealer(ctx, fb),
        Tile::Medkit => medkit(ctx, fb),
    }
}

pub fn floor(ctx: Ctx, fb: &mut FrameBuffer) {
    for offset in Size::new_u16(3, 3).coord_iter_row_major() {
        fb.set_cell_relative_to_ctx(
            ctx,
            offset,
            0,
            RenderCell::default()
                .with_character(' ')
                .with_background(colours::FLOOR_BACKGROUND),
        );
    }
    fb.set_cell_relative_to_ctx(
        ctx,
        Coord { x: 1, y: 1 },
        0,
        RenderCell::default()
            .with_character(' ')
            .with_background(colours::FLOOR_FOREGROUND),
    );
}

pub fn fuel_text_0(ctx: Ctx, fb: &mut FrameBuffer) {
    floor(ctx, fb);
    let style = Style::new()
        .with_bold(true)
        .with_foreground(colours::FUEL_BAY_FOREGROUND)
        .with_background(colours::FLOOR_BACKGROUND);
    let str_style = StrStyle::new(style);
    str_style.render("FUE", ctx.add_offset(Coord::new(0, 0)).add_depth(1), fb);
    str_style.render("BAY", ctx.add_offset(Coord::new(0, 1)).add_depth(1), fb);
    str_style.render("---", ctx.add_offset(Coord::new(0, 2)).add_depth(1), fb);
}

pub fn fuel_text_1(ctx: Ctx, fb: &mut FrameBuffer) {
    floor(ctx, fb);
    let style = Style::new()
        .with_bold(true)
        .with_foreground(colours::FUEL_BAY_FOREGROUND)
        .with_background(colours::FLOOR_BACKGROUND);
    let str_style = StrStyle::new(style);
    str_style.render("L  ", ctx.add_offset(Coord::new(0, 0)).add_depth(1), fb);
    str_style.render("   ", ctx.add_offset(Coord::new(0, 1)).add_depth(1), fb);
    str_style.render("->", ctx.add_offset(Coord::new(0, 2)).add_depth(1), fb);
}

pub fn fuel_hatch(ctx: Ctx, fb: &mut FrameBuffer) {
    for offset in Size::new_u16(3, 3).coord_iter_row_major() {
        fb.set_cell_relative_to_ctx(
            ctx,
            offset,
            0,
            RenderCell::default()
                .with_character(' ')
                .with_background(colours::FUEL_BAY_BACKGROUND),
        );
    }
    fb.set_cell_relative_to_ctx(
        ctx,
        Coord { x: 1, y: 1 },
        1,
        RenderCell::default()
            .with_character('●')
            .with_background(colours::FUEL_BAY_FOREGROUND),
    );
}

pub fn wall_top(ctx: Ctx, fb: &mut FrameBuffer) {
    for offset in Size::new_u16(3, 3).coord_iter_row_major() {
        fb.set_cell_relative_to_ctx(
            ctx,
            offset,
            0,
            RenderCell::default()
                .with_character(' ')
                .with_background(colours::WALL_TOP),
        );
    }
}

pub fn wall_front(ctx: Ctx, fb: &mut FrameBuffer) {
    for offset in Size::new_u16(3, 1).coord_iter_row_major() {
        fb.set_cell_relative_to_ctx(
            ctx,
            offset,
            0,
            RenderCell::default()
                .with_character(' ')
                .with_background(colours::WALL_TOP),
        );
    }
    for offset in Size::new_u16(3, 2).coord_iter_row_major() {
        fb.set_cell_relative_to_ctx(
            ctx,
            offset + Coord { x: 0, y: 1 },
            0,
            RenderCell::default()
                .with_character(' ')
                .with_background(colours::WALL_FRONT),
        );
    }
    for offset in Size::new_u16(3, 1).coord_iter_row_major() {
        fb.set_cell_relative_to_ctx(
            ctx,
            offset + Coord { x: 0, y: 1 },
            0,
            RenderCell::default()
                .with_character('▄')
                .with_foreground(colours::STRIPE),
        );
        fb.set_cell_relative_to_ctx(
            ctx,
            offset + Coord { x: 0, y: 2 },
            0,
            RenderCell::default()
                .with_character('▀')
                .with_foreground(colours::STRIPE),
        );
    }
}

pub fn wall_front_0(ctx: Ctx, fb: &mut FrameBuffer) {
    wall_front(ctx, fb);
    let blood = Style::new().with_bold(true).with_foreground(colours::BLOOD);
    let str_style = StrStyle::new(blood);
    str_style.render("DON", ctx.add_offset(Coord::new(0, 1)).add_depth(20), fb);
    str_style.render("DEA", ctx.add_offset(Coord::new(0, 2)).add_depth(20), fb);
}
pub fn wall_front_1(ctx: Ctx, fb: &mut FrameBuffer) {
    wall_front(ctx, fb);
    let blood = Style::new().with_bold(true).with_foreground(colours::BLOOD);
    let str_style = StrStyle::new(blood);
    str_style.render("'T ", ctx.add_offset(Coord::new(0, 1)).add_depth(20), fb);
    str_style.render("D I", ctx.add_offset(Coord::new(0, 2)).add_depth(20), fb);
}
pub fn wall_front_2(ctx: Ctx, fb: &mut FrameBuffer) {
    wall_front(ctx, fb);
    let blood = Style::new().with_bold(true).with_foreground(colours::BLOOD);
    let str_style = StrStyle::new(blood);
    str_style.render("OPE", ctx.add_offset(Coord::new(0, 1)).add_depth(20), fb);
    str_style.render("NSI", ctx.add_offset(Coord::new(0, 2)).add_depth(20), fb);
}
pub fn wall_front_3(ctx: Ctx, fb: &mut FrameBuffer) {
    wall_front(ctx, fb);
    let blood = Style::new().with_bold(true).with_foreground(colours::BLOOD);
    let str_style = StrStyle::new(blood);
    str_style.render("N! ", ctx.add_offset(Coord::new(0, 1)).add_depth(20), fb);
    str_style.render("DE!", ctx.add_offset(Coord::new(0, 2)).add_depth(20), fb);
}

pub fn player(ctx: Ctx, fb: &mut FrameBuffer) {
    let bold = false;
    fb.set_cell_relative_to_ctx(
        ctx,
        Coord { x: 0, y: 0 },
        0,
        RenderCell::default()
            .with_character('▗')
            .with_foreground(colours::PLAYER)
            .with_bold(bold),
    );
    fb.set_cell_relative_to_ctx(
        ctx,
        Coord { x: 1, y: 0 },
        0,
        RenderCell::default()
            .with_character('▀')
            .with_foreground(colours::PLAYER)
            .with_bold(bold),
    );
    fb.set_cell_relative_to_ctx(
        ctx,
        Coord { x: 2, y: 0 },
        0,
        RenderCell::default()
            .with_character('▖')
            .with_foreground(colours::PLAYER)
            .with_bold(bold),
    );
    fb.set_cell_relative_to_ctx(
        ctx,
        Coord { x: 0, y: 1 },
        0,
        RenderCell::default()
            .with_character('▐')
            .with_foreground(colours::PLAYER)
            .with_bold(bold),
    );
    fb.set_cell_relative_to_ctx(
        ctx,
        Coord { x: 1, y: 1 },
        0,
        RenderCell::default()
            .with_character('▐')
            .with_foreground(colours::PLAYER)
            .with_bold(bold),
    );
    fb.set_cell_relative_to_ctx(
        ctx,
        Coord { x: 2, y: 1 },
        0,
        RenderCell::default()
            .with_character('▌')
            .with_foreground(colours::PLAYER)
            .with_bold(bold),
    );
    fb.set_cell_relative_to_ctx(
        ctx,
        Coord { x: 0, y: 2 },
        0,
        RenderCell::default()
            .with_character('▝')
            .with_foreground(colours::PLAYER)
            .with_bold(bold),
    );
    fb.set_cell_relative_to_ctx(
        ctx,
        Coord { x: 1, y: 2 },
        0,
        RenderCell::default()
            .with_character('▄')
            .with_foreground(colours::PLAYER)
            .with_bold(bold),
    );
    fb.set_cell_relative_to_ctx(
        ctx,
        Coord { x: 2, y: 2 },
        0,
        RenderCell::default()
            .with_character('▖')
            .with_foreground(colours::PLAYER)
            .with_bold(bold),
    );
}

pub fn window_y(floor_below: bool, ctx: Ctx, fb: &mut FrameBuffer) {
    for offset in Size::new_u16(3, 1).coord_iter_row_major() {
        fb.set_cell_relative_to_ctx(
            ctx,
            offset,
            0,
            RenderCell::default()
                .with_character(' ')
                .with_background(colours::WALL_TOP),
        );
    }
    for offset in Size::new_u16(3, 2).coord_iter_row_major() {
        fb.set_cell_relative_to_ctx(
            ctx,
            offset + Coord { x: 0, y: 1 },
            0,
            RenderCell::default()
                .with_character(' ')
                .with_background(colours::WALL_FRONT),
        );
    }
    if floor_below {
        for offset in Size::new_u16(3, 1).coord_iter_row_major() {
            fb.set_cell_relative_to_ctx(
                ctx,
                offset + Coord { x: 0, y: 0 },
                0,
                RenderCell::default()
                    .with_character('▄')
                    .with_foreground(colours::WALL_FRONT),
            );
        }
        for offset in Size::new_u16(3, 1).coord_iter_row_major() {
            fb.set_cell_relative_to_ctx(
                ctx,
                offset + Coord { x: 0, y: 2 },
                0,
                RenderCell::default()
                    .with_character('▄')
                    .with_foreground(colours::FLOOR_BACKGROUND),
            );
        }
        fb.set_cell_relative_to_ctx(
            ctx,
            Coord { x: 1, y: 1 },
            1,
            RenderCell::default()
                .with_character(' ')
                .with_background(colours::WINDOWS),
        );
        fb.set_cell_relative_to_ctx(
            ctx,
            Coord { x: 0, y: 1 },
            0,
            RenderCell::default()
                .with_character('▌')
                .with_background(colours::WINDOWS)
                .with_foreground(colours::WALL_FRONT),
        );
        fb.set_cell_relative_to_ctx(
            ctx,
            Coord { x: 2, y: 1 },
            0,
            RenderCell::default()
                .with_character('▌')
                .with_background(colours::WALL_FRONT)
                .with_foreground(colours::WINDOWS),
        );
    } else {
        for offset in Size::new_u16(3, 1).coord_iter_row_major() {
            fb.set_cell_relative_to_ctx(
                ctx,
                offset + Coord { x: 0, y: 0 },
                0,
                RenderCell::default()
                    .with_character('▀')
                    .with_foreground(colours::FLOOR_BACKGROUND),
            );
        }
        fb.set_cell_relative_to_ctx(
            ctx,
            Coord { x: 1, y: 1 },
            0,
            RenderCell::default()
                .with_character('▄')
                .with_foreground(colours::WINDOWS),
        );
        fb.set_cell_relative_to_ctx(
            ctx,
            Coord { x: 1, y: 2 },
            0,
            RenderCell::default()
                .with_character('▀')
                .with_foreground(colours::WINDOWS),
        );
        fb.set_cell_relative_to_ctx(
            ctx,
            Coord { x: 0, y: 1 },
            0,
            RenderCell::default()
                .with_character('▗')
                .with_foreground(colours::WINDOWS),
        );
        fb.set_cell_relative_to_ctx(
            ctx,
            Coord { x: 2, y: 1 },
            0,
            RenderCell::default()
                .with_character('▖')
                .with_foreground(colours::WINDOWS),
        );
        fb.set_cell_relative_to_ctx(
            ctx,
            Coord { x: 0, y: 2 },
            0,
            RenderCell::default()
                .with_character('▝')
                .with_foreground(colours::WINDOWS),
        );
        fb.set_cell_relative_to_ctx(
            ctx,
            Coord { x: 2, y: 2 },
            0,
            RenderCell::default()
                .with_character('▘')
                .with_foreground(colours::WINDOWS),
        );
    }
}

pub fn window_x(ctx: Ctx, fb: &mut FrameBuffer) {
    for offset in Size::new_u16(3, 3).coord_iter_row_major() {
        fb.set_cell_relative_to_ctx(
            ctx,
            offset,
            0,
            RenderCell::default()
                .with_character(' ')
                .with_background(colours::WALL_TOP),
        );
    }
    fb.set_cell_relative_to_ctx(
        ctx,
        Coord { x: 1, y: 1 },
        0,
        RenderCell::default()
            .with_character(' ')
            .with_background(colours::WINDOWS),
    );
    fb.set_cell_relative_to_ctx(
        ctx,
        Coord { x: 0, y: 1 },
        0,
        RenderCell::default()
            .with_character('▌')
            .with_background(colours::WINDOWS)
            .with_foreground(colours::WALL_TOP),
    );
    fb.set_cell_relative_to_ctx(
        ctx,
        Coord { x: 2, y: 1 },
        0,
        RenderCell::default()
            .with_character('▌')
            .with_background(colours::WALL_TOP)
            .with_foreground(colours::WINDOWS),
    );
    fb.set_cell_relative_to_ctx(
        ctx,
        Coord { x: 0, y: 2 },
        0,
        RenderCell::default()
            .with_character('▝')
            .with_foreground(colours::WALL_FRONT),
    );
    fb.set_cell_relative_to_ctx(
        ctx,
        Coord { x: 2, y: 2 },
        0,
        RenderCell::default()
            .with_character('▘')
            .with_foreground(colours::WALL_FRONT),
    );
    fb.set_cell_relative_to_ctx(
        ctx,
        Coord { x: 1, y: 2 },
        0,
        RenderCell::default()
            .with_character('▀')
            .with_foreground(colours::WALL_FRONT),
    );
}

pub fn door_closed_y(ctx: Ctx, fb: &mut FrameBuffer) {
    for offset in Size::new_u16(3, 1).coord_iter_row_major() {
        fb.set_cell_relative_to_ctx(
            ctx,
            offset + Coord { x: 0, y: 1 },
            0,
            RenderCell::default()
                .with_character(' ')
                .with_background(colours::DOOR),
        );
        fb.set_cell_relative_to_ctx(
            ctx,
            offset + Coord { x: 0, y: 0 },
            0,
            RenderCell::default()
                .with_character('▄')
                .with_foreground(colours::DOOR_BORDER)
                .with_background(colours::FLOOR_BACKGROUND),
        );
        fb.set_cell_relative_to_ctx(
            ctx,
            offset + Coord { x: 0, y: 2 },
            0,
            RenderCell::default()
                .with_character('▄')
                .with_foreground(colours::FLOOR_BACKGROUND)
                .with_background(colours::DOOR_BORDER),
        );
    }
    fb.set_cell_relative_to_ctx(
        ctx,
        Coord { x: 0, y: 1 },
        0,
        RenderCell::default()
            .with_character('▌')
            .with_foreground(colours::DOOR_BORDER)
            .with_background(colours::DOOR),
    );
    fb.set_cell_relative_to_ctx(
        ctx,
        Coord { x: 2, y: 1 },
        0,
        RenderCell::default()
            .with_character('▌')
            .with_foreground(colours::DOOR)
            .with_background(colours::DOOR_BORDER),
    );
    fb.set_cell_relative_to_ctx(
        ctx,
        Coord { x: 1, y: 1 },
        0,
        RenderCell::default()
            .with_character('│')
            .with_foreground(colours::DOOR_BORDER)
            .with_bold(true),
    );
}

pub fn door_closed_x(ctx: Ctx, fb: &mut FrameBuffer) {
    for offset in Size::new_u16(1, 3).coord_iter_row_major() {
        fb.set_cell_relative_to_ctx(
            ctx,
            offset + Coord { x: 1, y: 0 },
            0,
            RenderCell::default()
                .with_character(' ')
                .with_background(colours::DOOR),
        );
        fb.set_cell_relative_to_ctx(
            ctx,
            offset + Coord { x: 0, y: 0 },
            0,
            RenderCell::default()
                .with_character(' ')
                .with_background(colours::FLOOR_BACKGROUND),
        );
        fb.set_cell_relative_to_ctx(
            ctx,
            offset + Coord { x: 0, y: 0 },
            0,
            RenderCell::default()
                .with_character('▌')
                .with_background(colours::DOOR_BORDER)
                .with_foreground(colours::FLOOR_BACKGROUND),
        );
        fb.set_cell_relative_to_ctx(
            ctx,
            offset + Coord { x: 2, y: 0 },
            0,
            RenderCell::default()
                .with_character('▌')
                .with_background(colours::FLOOR_BACKGROUND)
                .with_foreground(colours::DOOR_BORDER),
        );
    }
    fb.set_cell_relative_to_ctx(
        ctx,
        Coord { x: 1, y: 1 },
        0,
        RenderCell::default()
            .with_character('─')
            .with_foreground(colours::DOOR_BORDER)
            .with_bold(true),
    );
    fb.set_cell_relative_to_ctx(
        ctx,
        Coord { x: 1, y: 0 },
        0,
        RenderCell::default()
            .with_character('▀')
            .with_foreground(colours::DOOR_BORDER),
    );
    fb.set_cell_relative_to_ctx(
        ctx,
        Coord { x: 1, y: 2 },
        0,
        RenderCell::default()
            .with_character('▄')
            .with_foreground(colours::DOOR_BORDER),
    );
}

pub fn door_open_y(ctx: Ctx, fb: &mut FrameBuffer) {
    fb.set_cell_relative_to_ctx(
        ctx,
        Coord { x: 0, y: 1 },
        0,
        RenderCell::default()
            .with_character('▌')
            .with_foreground(colours::DOOR_BORDER),
    );
    fb.set_cell_relative_to_ctx(
        ctx,
        Coord { x: 2, y: 1 },
        0,
        RenderCell::default()
            .with_character('▐')
            .with_foreground(colours::DOOR_BORDER),
    );
    fb.set_cell_relative_to_ctx(
        ctx,
        Coord { x: 2, y: 0 },
        0,
        RenderCell::default()
            .with_character('▗')
            .with_foreground(colours::DOOR_BORDER),
    );
    fb.set_cell_relative_to_ctx(
        ctx,
        Coord { x: 0, y: 0 },
        0,
        RenderCell::default()
            .with_character('▖')
            .with_foreground(colours::DOOR_BORDER),
    );
    fb.set_cell_relative_to_ctx(
        ctx,
        Coord { x: 2, y: 2 },
        0,
        RenderCell::default()
            .with_character('▝')
            .with_foreground(colours::DOOR_BORDER),
    );
    fb.set_cell_relative_to_ctx(
        ctx,
        Coord { x: 0, y: 2 },
        0,
        RenderCell::default()
            .with_character('▘')
            .with_foreground(colours::DOOR_BORDER),
    );
}

pub fn door_open_x(ctx: Ctx, fb: &mut FrameBuffer) {
    fb.set_cell_relative_to_ctx(
        ctx,
        Coord { x: 2, y: 0 },
        0,
        RenderCell::default()
            .with_character('▘')
            .with_foreground(colours::DOOR_BORDER),
    );
    fb.set_cell_relative_to_ctx(
        ctx,
        Coord { x: 0, y: 0 },
        0,
        RenderCell::default()
            .with_character('▝')
            .with_foreground(colours::DOOR_BORDER),
    );
    fb.set_cell_relative_to_ctx(
        ctx,
        Coord { x: 2, y: 2 },
        0,
        RenderCell::default()
            .with_character('▖')
            .with_foreground(colours::DOOR_BORDER),
    );
    fb.set_cell_relative_to_ctx(
        ctx,
        Coord { x: 0, y: 2 },
        0,
        RenderCell::default()
            .with_character('▗')
            .with_foreground(colours::DOOR_BORDER),
    );
    fb.set_cell_relative_to_ctx(
        ctx,
        Coord { x: 1, y: 0 },
        0,
        RenderCell::default()
            .with_character('▀')
            .with_foreground(colours::DOOR_BORDER),
    );
    fb.set_cell_relative_to_ctx(
        ctx,
        Coord { x: 1, y: 2 },
        0,
        RenderCell::default()
            .with_character('▄')
            .with_foreground(colours::DOOR_BORDER),
    );
}

pub fn stairs(ctx: Ctx, fb: &mut FrameBuffer) {
    for offset in Size::new_u16(3, 3).coord_iter_row_major() {
        fb.set_cell_relative_to_ctx(
            ctx,
            offset,
            0,
            RenderCell::default()
                .with_character(' ')
                .with_background(colours::STAIRS_BACKGROUND),
        );
    }
    for offset in Size::new_u16(1, 3).coord_iter_row_major() {
        fb.set_cell_relative_to_ctx(
            ctx,
            offset,
            0,
            RenderCell::default()
                .with_character(' ')
                .with_background(colours::STAIRS_0),
        );
    }
    for offset in Size::new_u16(1, 2).coord_iter_row_major() {
        fb.set_cell_relative_to_ctx(
            ctx,
            offset + Coord { x: 1, y: 1 },
            0,
            RenderCell::default()
                .with_character(' ')
                .with_background(colours::STAIRS_1),
        );
    }
    fb.set_cell_relative_to_ctx(
        ctx,
        Coord { x: 2, y: 2 },
        0,
        RenderCell::default()
            .with_character(' ')
            .with_background(colours::STAIRS_2),
    );
}

pub fn zombie(entity: &ToRenderEntity, ctx: Ctx, fb: &mut FrameBuffer) {
    StrStyle::new(
        Style::new()
            .with_foreground(colours::ZOMBIE)
            .with_bold(true),
    )
    .render("Zmb", ctx, fb);
    StrStyle::new(
        Style::new()
            .with_foreground(colours::ZOMBIE)
            .with_bold(false),
    )
    .render(
        format!("♦{:02}", entity.armour.unwrap().value).as_str(),
        ctx.add_offset(Coord { x: 0, y: 1 }),
        fb,
    );
    StrStyle::new(
        Style::new()
            .with_foreground(colours::ZOMBIE)
            .with_bold(false),
    )
    .render(
        format!("♥{:02}", entity.hit_points.unwrap().current).as_str(),
        ctx.add_offset(Coord { x: 0, y: 2 }),
        fb,
    );
}

pub fn skeleton(entity: &ToRenderEntity, ctx: Ctx, fb: &mut FrameBuffer) {
    StrStyle::new(
        Style::new()
            .with_foreground(colours::SKELETON)
            .with_bold(true),
    )
    .render("Skl", ctx, fb);
    StrStyle::new(
        Style::new()
            .with_foreground(colours::SKELETON)
            .with_bold(false),
    )
    .render(
        format!("♦{:02}", entity.armour.unwrap().value).as_str(),
        ctx.add_offset(Coord { x: 0, y: 1 }),
        fb,
    );
    StrStyle::new(
        Style::new()
            .with_foreground(colours::SKELETON)
            .with_bold(false),
    )
    .render(
        format!("♥{:02}", entity.hit_points.unwrap().current).as_str(),
        ctx.add_offset(Coord { x: 0, y: 2 }),
        fb,
    );
}

pub fn skeleton_respawn(entity: &ToRenderEntity, ctx: Ctx, fb: &mut FrameBuffer) {
    StrStyle::new(
        Style::new()
            .with_foreground(colours::SKELETON)
            .with_bold(true),
    )
    .render("Res", ctx, fb);
    StrStyle::new(
        Style::new()
            .with_foreground(colours::SKELETON)
            .with_bold(true),
    )
    .render("paw", ctx.add_offset(Coord { x: 0, y: 1 }), fb);
    StrStyle::new(
        Style::new()
            .with_foreground(colours::SKELETON)
            .with_bold(true),
    )
    .render(
        format!("n{:02}", entity.skeleton_respawn.unwrap()).as_str(),
        ctx.add_offset(Coord { x: 0, y: 2 }),
        fb,
    );
}

pub fn boomer(entity: &ToRenderEntity, ctx: Ctx, fb: &mut FrameBuffer) {
    StrStyle::new(
        Style::new()
            .with_foreground(colours::BOOMER)
            .with_bold(true),
    )
    .render("Bmr", ctx, fb);
    StrStyle::new(
        Style::new()
            .with_foreground(colours::BOOMER)
            .with_bold(false),
    )
    .render(
        format!("♦{:02}", entity.armour.unwrap().value).as_str(),
        ctx.add_offset(Coord { x: 0, y: 1 }),
        fb,
    );
    StrStyle::new(
        Style::new()
            .with_foreground(colours::BOOMER)
            .with_bold(false),
    )
    .render(
        format!("♥{:02}", entity.hit_points.unwrap().current).as_str(),
        ctx.add_offset(Coord { x: 0, y: 2 }),
        fb,
    );
}

pub fn tank(entity: &ToRenderEntity, ctx: Ctx, fb: &mut FrameBuffer) {
    StrStyle::new(Style::new().with_foreground(colours::TANK).with_bold(true))
        .render("Tnk", ctx, fb);
    StrStyle::new(Style::new().with_foreground(colours::TANK).with_bold(false)).render(
        format!("♦{:02}", entity.armour.unwrap().value).as_str(),
        ctx.add_offset(Coord { x: 0, y: 1 }),
        fb,
    );
    StrStyle::new(Style::new().with_foreground(colours::TANK).with_bold(false)).render(
        format!("♥{:02}", entity.hit_points.unwrap().current).as_str(),
        ctx.add_offset(Coord { x: 0, y: 2 }),
        fb,
    );
}

pub fn bullet(ctx: Ctx, fb: &mut FrameBuffer) {
    fb.set_cell_relative_to_ctx(
        ctx,
        Coord { x: 1, y: 1 },
        1,
        RenderCell::default()
            .with_character(' ')
            .with_background(colours::BULLET),
    );
}

pub fn credit1(ctx: Ctx, fb: &mut FrameBuffer) {
    let str_style = StrStyle::new(
        Style::new()
            .with_foreground(colours::CREDIT_FOREGROUND)
            .with_bold(true),
    );
    str_style.render("$1 ", ctx, fb);
    str_style.render("CRE", ctx.add_offset(Coord { x: 0, y: 1 }), fb);
    str_style.render("DIT", ctx.add_offset(Coord { x: 0, y: 2 }), fb);
}

pub fn credit2(ctx: Ctx, fb: &mut FrameBuffer) {
    let str_style = StrStyle::new(
        Style::new()
            .with_foreground(colours::CREDIT_FOREGROUND)
            .with_bold(true),
    );
    str_style.render("$2.", ctx, fb);
    str_style.render("CRE", ctx.add_offset(Coord { x: 0, y: 1 }), fb);
    str_style.render("DIT", ctx.add_offset(Coord { x: 0, y: 2 }), fb);
}

pub fn upgrade(ctx: Ctx, fb: &mut FrameBuffer) {
    let str_style = StrStyle::new(
        Style::new()
            .with_foreground(colours::UPGRADE_FOREGROUND)
            .with_background(colours::UPGRADE_BACKGROUND)
            .with_bold(true),
    );
    str_style.render("UPG", ctx, fb);
    str_style.render("RAD", ctx.add_offset(Coord { x: 0, y: 1 }), fb);
    str_style.render("E++", ctx.add_offset(Coord { x: 0, y: 2 }), fb);
}

pub fn map_locked(ctx: Ctx, fb: &mut FrameBuffer) {
    let str_style = StrStyle::new(
        Style::new()
            .with_foreground(colours::MAP_FOREGROUND)
            .with_background(colours::MAP_BACKGROUND)
            .with_bold(true),
    );
    str_style.render("***", ctx, fb);
    str_style.render("MAP", ctx.add_offset(Coord { x: 0, y: 1 }), fb);
    str_style.render("***", ctx.add_offset(Coord { x: 0, y: 2 }), fb);
}

pub fn map(ctx: Ctx, fb: &mut FrameBuffer) {
    let str_style = StrStyle::new(
        Style::new()
            .with_foreground(colours::MAP_FOREGROUND)
            .with_background(colours::MAP_BACKGROUND)
            .with_bold(true),
    );
    str_style.render("   ", ctx, fb);
    str_style.render("MAP", ctx.add_offset(Coord { x: 0, y: 1 }), fb);
    str_style.render("   ", ctx.add_offset(Coord { x: 0, y: 2 }), fb);
}

pub fn chainsaw(ctx: Ctx, fb: &mut FrameBuffer) {
    fb.set_cell_relative_to_ctx(
        ctx,
        Coord { x: 0, y: 0 },
        0,
        RenderCell::default()
            .with_character('╥')
            .with_foreground(colours::GUN_METAL),
    );
    fb.set_cell_relative_to_ctx(
        ctx,
        Coord { x: 0, y: 1 },
        0,
        RenderCell::default()
            .with_character('-')
            .with_foreground(colours::GUN_METAL)
            .with_background(colours::CHAINSAW),
    );
    fb.set_cell_relative_to_ctx(
        ctx,
        Coord { x: 1, y: 1 },
        0,
        RenderCell::default()
            .with_character(' ')
            .with_background(colours::GUN_METAL),
    );
    fb.set_cell_relative_to_ctx(
        ctx,
        Coord { x: 2, y: 1 },
        0,
        RenderCell::default()
            .with_character('►')
            .with_foreground(colours::GUN_METAL),
    );
}

pub fn shotgun(ctx: Ctx, fb: &mut FrameBuffer) {
    fb.set_cell_relative_to_ctx(
        ctx,
        Coord { x: 0, y: 2 },
        0,
        RenderCell::default()
            .with_character('▘')
            .with_foreground(colours::WOOD),
    );
    fb.set_cell_relative_to_ctx(
        ctx,
        Coord { x: 0, y: 1 },
        0,
        RenderCell::default()
            .with_character('▖')
            .with_foreground(colours::WOOD)
            .with_background(colours::GUN_METAL),
    );
    fb.set_cell_relative_to_ctx(
        ctx,
        Coord { x: 1, y: 1 },
        0,
        RenderCell::default()
            .with_character('▀')
            .with_foreground(colours::GUN_METAL)
            .with_background(colours::WOOD),
    );
    fb.set_cell_relative_to_ctx(
        ctx,
        Coord { x: 2, y: 1 },
        0,
        RenderCell::default()
            .with_character('▀')
            .with_foreground(colours::GUN_METAL),
    );
}

pub fn railgun(ctx: Ctx, fb: &mut FrameBuffer) {
    fb.set_cell_relative_to_ctx(
        ctx,
        Coord { x: 0, y: 2 },
        0,
        RenderCell::default()
            .with_character('▘')
            .with_foreground(colours::GUN_METAL),
    );
    fb.set_cell_relative_to_ctx(
        ctx,
        Coord { x: 0, y: 1 },
        0,
        RenderCell::default()
            .with_character('-')
            .with_background(colours::GUN_METAL)
            .with_foreground(colours::PLASMA),
    );
    fb.set_cell_relative_to_ctx(
        ctx,
        Coord { x: 1, y: 1 },
        0,
        RenderCell::default()
            .with_character('=')
            .with_background(colours::GUN_METAL)
            .with_foreground(colours::PLASMA),
    );
    fb.set_cell_relative_to_ctx(
        ctx,
        Coord { x: 2, y: 1 },
        0,
        RenderCell::default()
            .with_character('=')
            .with_background(colours::GUN_METAL)
            .with_foreground(colours::PLASMA),
    );
}

pub fn rifle(ctx: Ctx, fb: &mut FrameBuffer) {
    fb.set_cell_relative_to_ctx(
        ctx,
        Coord { x: 0, y: 2 },
        0,
        RenderCell::default()
            .with_character('▘')
            .with_foreground(colours::GUN_METAL),
    );
    fb.set_cell_relative_to_ctx(
        ctx,
        Coord { x: 1, y: 2 },
        0,
        RenderCell::default()
            .with_character('▘')
            .with_foreground(colours::GUN_METAL),
    );
    fb.set_cell_relative_to_ctx(
        ctx,
        Coord { x: 0, y: 1 },
        0,
        RenderCell::default()
            .with_character(' ')
            .with_background(colours::GUN_METAL),
    );
    fb.set_cell_relative_to_ctx(
        ctx,
        Coord { x: 1, y: 1 },
        0,
        RenderCell::default()
            .with_character('▗')
            .with_foreground(colours::LASER)
            .with_background(colours::GUN_METAL),
    );
    fb.set_cell_relative_to_ctx(
        ctx,
        Coord { x: 2, y: 1 },
        0,
        RenderCell::default()
            .with_character('▀')
            .with_foreground(colours::GUN_METAL),
    );
    fb.set_cell_relative_to_ctx(
        ctx,
        Coord { x: 2, y: 0 },
        0,
        RenderCell::default()
            .with_character('▗')
            .with_foreground(colours::GUN_METAL),
    );
}

pub fn gaus_cannon(ctx: Ctx, fb: &mut FrameBuffer) {
    fb.set_cell_relative_to_ctx(
        ctx,
        Coord { x: 0, y: 2 },
        0,
        RenderCell::default()
            .with_character('▘')
            .with_foreground(colours::GUN_METAL),
    );
    fb.set_cell_relative_to_ctx(
        ctx,
        Coord { x: 0, y: 1 },
        0,
        RenderCell::default()
            .with_character('▌')
            .with_background(colours::GUN_METAL)
            .with_foreground(colours::GAUS),
    );
    fb.set_cell_relative_to_ctx(
        ctx,
        Coord { x: 1, y: 1 },
        0,
        RenderCell::default()
            .with_character('▌')
            .with_background(colours::GUN_METAL)
            .with_foreground(colours::GAUS),
    );
    fb.set_cell_relative_to_ctx(
        ctx,
        Coord { x: 2, y: 1 },
        0,
        RenderCell::default()
            .with_character('▌')
            .with_background(colours::GUN_METAL)
            .with_foreground(colours::GAUS),
    );
    fb.set_cell_relative_to_ctx(
        ctx,
        Coord { x: 2, y: 0 },
        0,
        RenderCell::default()
            .with_character('▗')
            .with_foreground(colours::GUN_METAL),
    );
    fb.set_cell_relative_to_ctx(
        ctx,
        Coord { x: 2, y: 2 },
        0,
        RenderCell::default()
            .with_character('▝')
            .with_foreground(colours::GUN_METAL),
    );
}

pub fn oxidiser(ctx: Ctx, fb: &mut FrameBuffer) {
    fb.set_cell_relative_to_ctx(
        ctx,
        Coord { x: 0, y: 0 },
        0,
        RenderCell::default()
            .with_character('┌')
            .with_foreground(colours::GUN_METAL),
    );
    fb.set_cell_relative_to_ctx(
        ctx,
        Coord { x: 1, y: 0 },
        0,
        RenderCell::default()
            .with_character('┬')
            .with_foreground(colours::GUN_METAL),
    );
    fb.set_cell_relative_to_ctx(
        ctx,
        Coord { x: 2, y: 0 },
        0,
        RenderCell::default()
            .with_character('┐')
            .with_foreground(colours::GUN_METAL),
    );
    fb.set_cell_relative_to_ctx(
        ctx,
        Coord { x: 0, y: 2 },
        0,
        RenderCell::default()
            .with_character('▘')
            .with_foreground(colours::GUN_METAL),
    );
    fb.set_cell_relative_to_ctx(
        ctx,
        Coord { x: 0, y: 1 },
        0,
        RenderCell::default()
            .with_character('●')
            .with_foreground(colours::OXYGEN)
            .with_background(colours::GUN_METAL),
    );
    fb.set_cell_relative_to_ctx(
        ctx,
        Coord { x: 1, y: 1 },
        0,
        RenderCell::default()
            .with_character('●')
            .with_foreground(colours::OXYGEN)
            .with_background(colours::GUN_METAL),
    );
    fb.set_cell_relative_to_ctx(
        ctx,
        Coord { x: 2, y: 1 },
        0,
        RenderCell::default()
            .with_character('●')
            .with_foreground(colours::OXYGEN)
            .with_background(colours::GUN_METAL),
    );
}

pub fn life_stealer(ctx: Ctx, fb: &mut FrameBuffer) {
    fb.set_cell_relative_to_ctx(
        ctx,
        Coord { x: 1, y: 2 },
        0,
        RenderCell::default()
            .with_character('└')
            .with_foreground(colours::HEALTH),
    );
    fb.set_cell_relative_to_ctx(
        ctx,
        Coord { x: 2, y: 2 },
        0,
        RenderCell::default()
            .with_character('┘')
            .with_foreground(colours::HEALTH),
    );
    fb.set_cell_relative_to_ctx(
        ctx,
        Coord { x: 0, y: 2 },
        0,
        RenderCell::default()
            .with_character('▘')
            .with_foreground(colours::GUN_METAL),
    );
    fb.set_cell_relative_to_ctx(
        ctx,
        Coord { x: 0, y: 1 },
        0,
        RenderCell::default()
            .with_character('♥')
            .with_foreground(colours::HEALTH)
            .with_background(colours::GUN_METAL),
    );
    fb.set_cell_relative_to_ctx(
        ctx,
        Coord { x: 1, y: 1 },
        0,
        RenderCell::default()
            .with_character('♥')
            .with_foreground(colours::HEALTH)
            .with_background(colours::GUN_METAL),
    );
    fb.set_cell_relative_to_ctx(
        ctx,
        Coord { x: 2, y: 1 },
        0,
        RenderCell::default()
            .with_character('♥')
            .with_foreground(colours::HEALTH)
            .with_background(colours::GUN_METAL),
    );
}

pub fn medkit(ctx: Ctx, fb: &mut FrameBuffer) {
    for coord in Size::new_u16(3, 2).coord_iter_row_major() {
        fb.set_cell_relative_to_ctx(
            ctx,
            coord + Coord { x: 0, y: 1 },
            0,
            RenderCell::default()
                .with_character(' ')
                .with_background(colours::MEDKIT),
        );
    }
    fb.set_cell_relative_to_ctx(
        ctx,
        Coord { x: 1, y: 2 },
        0,
        RenderCell::default()
            .with_bold(true)
            .with_character('+')
            .with_foreground(colours::HEALTH),
    );
    fb.set_cell_relative_to_ctx(
        ctx,
        Coord { x: 2, y: 1 },
        0,
        RenderCell::default()
            .with_character('▌')
            .with_foreground(colours::MEDKIT_TOP),
    );
    fb.set_cell_relative_to_ctx(
        ctx,
        Coord { x: 0, y: 1 },
        0,
        RenderCell::default()
            .with_character('▐')
            .with_foreground(colours::MEDKIT_TOP),
    );
    fb.set_cell_relative_to_ctx(
        ctx,
        Coord { x: 1, y: 1 },
        0,
        RenderCell::default()
            .with_character(' ')
            .with_background(colours::MEDKIT_TOP),
    );
}
