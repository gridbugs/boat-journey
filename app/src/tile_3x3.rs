use crate::colours;
use chargrid::render::{
    grid_2d::coord_2d::Axis, ColModify, Coord, Frame, Size, Style, View, ViewCell, ViewContext,
};
use chargrid::text::StringViewSingleLine;
use orbital_decay_game::{EntityTile, Game, Tile, ToRenderEntity, VisibilityCell, VisibilityGrid};

pub const OFFSETS: [Coord; 9] = [
    Coord::new(0, 0),
    Coord::new(0, 1),
    Coord::new(0, 2),
    Coord::new(1, 0),
    Coord::new(1, 1),
    Coord::new(1, 2),
    Coord::new(2, 0),
    Coord::new(2, 1),
    Coord::new(2, 2),
];

pub fn render_3x3_from_visibility<F: Frame, C: ColModify>(
    coord: Coord,
    visibility_cell: &VisibilityCell,
    game: &Game,
    view_context: ViewContext<C>,
    frame: &mut F,
) {
    let view_context = view_context.add_offset(coord * 3);
    let mut render_tile = |entity, tile, view_context| match tile {
        Tile::Wall => {
            let below = coord + Coord::new(0, 1);
            if game.contains_wall_like(below) {
                wall_top(view_context, frame);
            } else {
                wall_front(view_context, frame);
            }
        }
        Tile::WallText0 => wall_front_0(view_context, frame),
        Tile::WallText1 => wall_front_1(view_context, frame),
        Tile::WallText2 => wall_front_2(view_context, frame),
        Tile::WallText3 => wall_front_3(view_context, frame),
        Tile::Floor => floor(view_context, frame),
        Tile::FuelText0 => fuel_text_0(view_context, frame),
        Tile::FuelText1 => fuel_text_1(view_context, frame),
        Tile::FuelHatch => fuel_hatch(view_context, frame),
        Tile::Player => player(view_context, frame),
        Tile::Window(Axis::Y) => {
            let below = coord + Coord::new(0, 1);
            window_y(game.contains_floor(below), view_context, frame);
        }
        Tile::Window(Axis::X) => window_x(view_context, frame),
        Tile::DoorOpen(Axis::X) => door_open_x(view_context, frame),
        Tile::DoorOpen(Axis::Y) => door_open_y(view_context, frame),
        Tile::DoorClosed(Axis::X) => door_closed_x(view_context, frame),
        Tile::DoorClosed(Axis::Y) => door_closed_y(view_context, frame),
        Tile::Stairs => stairs(view_context, frame),
        Tile::Bullet => bullet(view_context, frame),
        Tile::Zombie => {
            if let Some(entity) = game.to_render_entity(entity) {
                zombie(&entity, view_context, frame);
            }
        }
    };
    let tile_layers = visibility_cell.tile_layers();
    if let Some(EntityTile { entity, tile }) = tile_layers.floor {
        render_tile(entity, tile, view_context.add_depth(0));
    }
    if let Some(EntityTile { entity, tile }) = tile_layers.feature {
        render_tile(entity, tile, view_context.add_depth(1));
    }
    if let Some(EntityTile { entity, tile }) = tile_layers.character {
        render_tile(entity, tile, view_context.add_depth(2));
    }
}

pub fn render_3x3_from_visibility_remembered<F: Frame, C: ColModify>(
    coord: Coord,
    visibility_cell: &VisibilityCell,
    game: &Game,
    view_context: ViewContext<C>,
    frame: &mut F,
) {
    let view_context = view_context.add_offset(coord * 3);
    let mut render_tile = |tile, view_context| match tile {
        Tile::Wall => {
            let below = coord + Coord::new(0, 1);
            if game.contains_wall_like(below) {
                wall_top(view_context, frame);
            } else {
                wall_front(view_context, frame);
            }
        }
        Tile::WallText0 => wall_front_0(view_context, frame),
        Tile::WallText1 => wall_front_1(view_context, frame),
        Tile::WallText2 => wall_front_2(view_context, frame),
        Tile::WallText3 => wall_front_3(view_context, frame),
        Tile::Floor => floor(view_context, frame),
        Tile::FuelText0 => fuel_text_0(view_context, frame),
        Tile::FuelText1 => fuel_text_1(view_context, frame),
        Tile::FuelHatch => fuel_hatch(view_context, frame),
        Tile::Player => player(view_context, frame),
        Tile::Window(Axis::Y) => {
            let below = coord + Coord::new(0, 1);
            window_y(game.contains_floor(below), view_context, frame);
        }
        Tile::Window(Axis::X) => window_x(view_context, frame),
        Tile::DoorOpen(Axis::X) => door_open_x(view_context, frame),
        Tile::DoorOpen(Axis::Y) => door_open_y(view_context, frame),
        Tile::DoorClosed(Axis::X) => door_closed_x(view_context, frame),
        Tile::DoorClosed(Axis::Y) => door_closed_y(view_context, frame),
        Tile::Stairs => stairs(view_context, frame),
        Tile::Bullet => bullet(view_context, frame),
        Tile::Zombie => (),
    };
    let tile_layers = visibility_cell.tile_layers();
    if let Some(EntityTile { entity: _, tile }) = tile_layers.floor {
        render_tile(tile, view_context.add_depth(0));
    }
    if let Some(EntityTile { entity: _, tile }) = tile_layers.feature {
        render_tile(tile, view_context.add_depth(1));
    }
    if let Some(EntityTile { entity: _, tile }) = tile_layers.character {
        render_tile(tile, view_context.add_depth(2));
    }
}

pub fn render_3x3_tile<F: Frame, C: ColModify>(
    coord: Coord,
    tile: Tile,
    view_context: ViewContext<C>,
    frame: &mut F,
) {
    let view_context = view_context.add_offset(coord * 3);
    match tile {
        Tile::Bullet => bullet(view_context, frame),
        _ => (),
    }
}

pub fn render_3x3<F: Frame, C: ColModify>(
    entity: &ToRenderEntity,
    game: &Game,
    view_context: ViewContext<C>,
    frame: &mut F,
) {
    let view_context = view_context.add_offset(entity.coord * 3);
    match entity.tile {
        Tile::Wall => {
            let below = entity.coord + Coord::new(0, 1);
            if game.contains_wall_like(below) {
                wall_top(view_context, frame);
            } else {
                wall_front(view_context, frame);
            }
        }
        Tile::WallText0 => wall_front_0(view_context, frame),
        Tile::WallText1 => wall_front_1(view_context, frame),
        Tile::WallText2 => wall_front_2(view_context, frame),
        Tile::WallText3 => wall_front_3(view_context, frame),
        Tile::Floor => floor(view_context, frame),
        Tile::FuelText0 => fuel_text_0(view_context, frame),
        Tile::FuelText1 => fuel_text_1(view_context, frame),
        Tile::FuelHatch => fuel_hatch(view_context, frame),
        Tile::Player => player(view_context, frame),
        Tile::Window(Axis::Y) => {
            let below = entity.coord + Coord::new(0, 1);
            window_y(game.contains_floor(below), view_context, frame);
        }
        Tile::Window(Axis::X) => window_x(view_context, frame),
        Tile::DoorOpen(Axis::X) => door_open_x(view_context, frame),
        Tile::DoorOpen(Axis::Y) => door_open_y(view_context, frame),
        Tile::DoorClosed(Axis::X) => door_closed_x(view_context, frame),
        Tile::DoorClosed(Axis::Y) => door_closed_y(view_context, frame),
        Tile::Stairs => stairs(view_context, frame),
        Tile::Bullet => bullet(view_context, frame),
        Tile::Zombie => zombie(entity, view_context, frame),
    }
}

pub fn floor<F: Frame, C: ColModify>(view_context: ViewContext<C>, frame: &mut F) {
    for offset in Size::new_u16(3, 3).coord_iter_row_major() {
        frame.set_cell_relative(
            offset,
            0,
            ViewCell::new()
                .with_character(' ')
                .with_background(colours::FLOOR_BACKGROUND),
            view_context,
        );
    }
    frame.set_cell_relative(
        Coord { x: 1, y: 1 },
        1,
        ViewCell::new()
            .with_character(' ')
            .with_background(colours::FLOOR_FOREGROUND),
        view_context,
    );
}

pub fn fuel_text_0<F: Frame, C: ColModify>(view_context: ViewContext<C>, frame: &mut F) {
    floor(view_context, frame);
    let style = Style::new()
        .with_bold(true)
        .with_foreground(colours::FUEL_BAY_FOREGROUND)
        .with_background(colours::FLOOR_BACKGROUND);
    let mut view = StringViewSingleLine::new(style);
    view.view(
        "FUE",
        view_context.add_offset(Coord::new(0, 0)).add_depth(1),
        frame,
    );
    view.view(
        "BAY",
        view_context.add_offset(Coord::new(0, 1)).add_depth(1),
        frame,
    );
    view.view(
        "---",
        view_context.add_offset(Coord::new(0, 2)).add_depth(1),
        frame,
    );
}

pub fn fuel_text_1<F: Frame, C: ColModify>(view_context: ViewContext<C>, frame: &mut F) {
    floor(view_context, frame);
    let style = Style::new()
        .with_bold(true)
        .with_foreground(colours::FUEL_BAY_FOREGROUND)
        .with_background(colours::FLOOR_BACKGROUND);
    let mut view = StringViewSingleLine::new(style);
    view.view(
        "L  ",
        view_context.add_offset(Coord::new(0, 0)).add_depth(1),
        frame,
    );
    view.view(
        "   ",
        view_context.add_offset(Coord::new(0, 1)).add_depth(1),
        frame,
    );
    view.view(
        "->",
        view_context.add_offset(Coord::new(0, 2)).add_depth(1),
        frame,
    );
}

pub fn fuel_hatch<F: Frame, C: ColModify>(view_context: ViewContext<C>, frame: &mut F) {
    for offset in Size::new_u16(3, 3).coord_iter_row_major() {
        frame.set_cell_relative(
            offset,
            0,
            ViewCell::new()
                .with_character(' ')
                .with_background(colours::FUEL_BAY_BACKGROUND),
            view_context,
        );
    }
    frame.set_cell_relative(
        Coord { x: 1, y: 1 },
        1,
        ViewCell::new()
            .with_character('●')
            .with_background(colours::FUEL_BAY_FOREGROUND),
        view_context,
    );
}

pub fn wall_top<F: Frame, C: ColModify>(view_context: ViewContext<C>, frame: &mut F) {
    for offset in Size::new_u16(3, 3).coord_iter_row_major() {
        frame.set_cell_relative(
            offset,
            0,
            ViewCell::new()
                .with_character(' ')
                .with_background(colours::WALL_TOP),
            view_context,
        );
    }
}

pub fn wall_front<F: Frame, C: ColModify>(view_context: ViewContext<C>, frame: &mut F) {
    for offset in Size::new_u16(3, 1).coord_iter_row_major() {
        frame.set_cell_relative(
            offset,
            0,
            ViewCell::new()
                .with_character(' ')
                .with_background(colours::WALL_TOP),
            view_context,
        );
    }
    for offset in Size::new_u16(3, 2).coord_iter_row_major() {
        frame.set_cell_relative(
            offset + Coord { x: 0, y: 1 },
            0,
            ViewCell::new()
                .with_character(' ')
                .with_background(colours::WALL_FRONT),
            view_context,
        );
    }
    for offset in Size::new_u16(3, 1).coord_iter_row_major() {
        frame.set_cell_relative(
            offset + Coord { x: 0, y: 1 },
            0,
            ViewCell::new()
                .with_character('▄')
                .with_foreground(colours::STRIPE),
            view_context,
        );
        frame.set_cell_relative(
            offset + Coord { x: 0, y: 2 },
            0,
            ViewCell::new()
                .with_character('▀')
                .with_foreground(colours::STRIPE),
            view_context,
        );
    }
}

pub fn wall_front_0<F: Frame, C: ColModify>(view_context: ViewContext<C>, frame: &mut F) {
    wall_front(view_context, frame);
    let blood = Style::new().with_bold(true).with_foreground(colours::BLOOD);
    let mut view = StringViewSingleLine::new(blood);
    view.view(
        "DON",
        view_context.add_offset(Coord::new(0, 1)).add_depth(20),
        frame,
    );
    view.view(
        "DEA",
        view_context.add_offset(Coord::new(0, 2)).add_depth(20),
        frame,
    );
}
pub fn wall_front_1<F: Frame, C: ColModify>(view_context: ViewContext<C>, frame: &mut F) {
    wall_front(view_context, frame);
    let blood = Style::new().with_bold(true).with_foreground(colours::BLOOD);
    let mut view = StringViewSingleLine::new(blood);
    view.view(
        "'T ",
        view_context.add_offset(Coord::new(0, 1)).add_depth(20),
        frame,
    );
    view.view(
        "D I",
        view_context.add_offset(Coord::new(0, 2)).add_depth(20),
        frame,
    );
}
pub fn wall_front_2<F: Frame, C: ColModify>(view_context: ViewContext<C>, frame: &mut F) {
    wall_front(view_context, frame);
    let blood = Style::new().with_bold(true).with_foreground(colours::BLOOD);
    let mut view = StringViewSingleLine::new(blood);
    view.view(
        "OPE",
        view_context.add_offset(Coord::new(0, 1)).add_depth(20),
        frame,
    );
    view.view(
        "NSI",
        view_context.add_offset(Coord::new(0, 2)).add_depth(20),
        frame,
    );
}
pub fn wall_front_3<F: Frame, C: ColModify>(view_context: ViewContext<C>, frame: &mut F) {
    wall_front(view_context, frame);
    let blood = Style::new().with_bold(true).with_foreground(colours::BLOOD);
    let mut view = StringViewSingleLine::new(blood);
    view.view(
        "N! ",
        view_context.add_offset(Coord::new(0, 1)).add_depth(20),
        frame,
    );
    view.view(
        "DE!",
        view_context.add_offset(Coord::new(0, 2)).add_depth(20),
        frame,
    );
}

pub fn player<F: Frame, C: ColModify>(view_context: ViewContext<C>, frame: &mut F) {
    let bold = false;
    frame.set_cell_relative(
        Coord { x: 0, y: 0 },
        0,
        ViewCell::new()
            .with_character('▗')
            .with_foreground(colours::PLAYER)
            .with_bold(bold),
        view_context,
    );
    frame.set_cell_relative(
        Coord { x: 1, y: 0 },
        0,
        ViewCell::new()
            .with_character('▀')
            .with_foreground(colours::PLAYER)
            .with_bold(bold),
        view_context,
    );
    frame.set_cell_relative(
        Coord { x: 2, y: 0 },
        0,
        ViewCell::new()
            .with_character('▖')
            .with_foreground(colours::PLAYER)
            .with_bold(bold),
        view_context,
    );
    frame.set_cell_relative(
        Coord { x: 0, y: 1 },
        0,
        ViewCell::new()
            .with_character('▐')
            .with_foreground(colours::PLAYER)
            .with_bold(bold),
        view_context,
    );
    frame.set_cell_relative(
        Coord { x: 1, y: 1 },
        0,
        ViewCell::new()
            .with_character('▐')
            .with_foreground(colours::PLAYER)
            .with_bold(bold),
        view_context,
    );
    frame.set_cell_relative(
        Coord { x: 2, y: 1 },
        0,
        ViewCell::new()
            .with_character('▌')
            .with_foreground(colours::PLAYER)
            .with_bold(bold),
        view_context,
    );
    frame.set_cell_relative(
        Coord { x: 0, y: 2 },
        0,
        ViewCell::new()
            .with_character('▝')
            .with_foreground(colours::PLAYER)
            .with_bold(bold),
        view_context,
    );
    frame.set_cell_relative(
        Coord { x: 1, y: 2 },
        0,
        ViewCell::new()
            .with_character('▄')
            .with_foreground(colours::PLAYER)
            .with_bold(bold),
        view_context,
    );
    frame.set_cell_relative(
        Coord { x: 2, y: 2 },
        0,
        ViewCell::new()
            .with_character('▖')
            .with_foreground(colours::PLAYER)
            .with_bold(bold),
        view_context,
    );
}

pub fn window_y<F: Frame, C: ColModify>(
    floor_below: bool,
    view_context: ViewContext<C>,
    frame: &mut F,
) {
    for offset in Size::new_u16(3, 1).coord_iter_row_major() {
        frame.set_cell_relative(
            offset,
            0,
            ViewCell::new()
                .with_character(' ')
                .with_background(colours::WALL_TOP),
            view_context,
        );
    }
    for offset in Size::new_u16(3, 2).coord_iter_row_major() {
        frame.set_cell_relative(
            offset + Coord { x: 0, y: 1 },
            0,
            ViewCell::new()
                .with_character(' ')
                .with_background(colours::WALL_FRONT),
            view_context,
        );
    }
    if floor_below {
        for offset in Size::new_u16(3, 1).coord_iter_row_major() {
            frame.set_cell_relative(
                offset + Coord { x: 0, y: 0 },
                0,
                ViewCell::new()
                    .with_character('▄')
                    .with_foreground(colours::WALL_FRONT),
                view_context,
            );
        }
        for offset in Size::new_u16(3, 1).coord_iter_row_major() {
            frame.set_cell_relative(
                offset + Coord { x: 0, y: 2 },
                0,
                ViewCell::new()
                    .with_character('▄')
                    .with_foreground(colours::FLOOR_BACKGROUND),
                view_context,
            );
        }
        frame.set_cell_relative(
            Coord { x: 1, y: 1 },
            1,
            ViewCell::new()
                .with_character(' ')
                .with_background(colours::WINDOWS),
            view_context,
        );
        frame.set_cell_relative(
            Coord { x: 0, y: 1 },
            0,
            ViewCell::new()
                .with_character('▌')
                .with_background(colours::WINDOWS)
                .with_foreground(colours::WALL_FRONT),
            view_context,
        );
        frame.set_cell_relative(
            Coord { x: 2, y: 1 },
            0,
            ViewCell::new()
                .with_character('▌')
                .with_background(colours::WALL_FRONT)
                .with_foreground(colours::WINDOWS),
            view_context,
        );
    } else {
        for offset in Size::new_u16(3, 1).coord_iter_row_major() {
            frame.set_cell_relative(
                offset + Coord { x: 0, y: 0 },
                0,
                ViewCell::new()
                    .with_character('▀')
                    .with_foreground(colours::FLOOR_BACKGROUND),
                view_context,
            );
        }
        frame.set_cell_relative(
            Coord { x: 1, y: 1 },
            0,
            ViewCell::new()
                .with_character('▄')
                .with_foreground(colours::WINDOWS),
            view_context,
        );
        frame.set_cell_relative(
            Coord { x: 1, y: 2 },
            0,
            ViewCell::new()
                .with_character('▀')
                .with_foreground(colours::WINDOWS),
            view_context,
        );
        frame.set_cell_relative(
            Coord { x: 0, y: 1 },
            0,
            ViewCell::new()
                .with_character('▗')
                .with_foreground(colours::WINDOWS),
            view_context,
        );
        frame.set_cell_relative(
            Coord { x: 2, y: 1 },
            0,
            ViewCell::new()
                .with_character('▖')
                .with_foreground(colours::WINDOWS),
            view_context,
        );
        frame.set_cell_relative(
            Coord { x: 0, y: 2 },
            0,
            ViewCell::new()
                .with_character('▝')
                .with_foreground(colours::WINDOWS),
            view_context,
        );
        frame.set_cell_relative(
            Coord { x: 2, y: 2 },
            0,
            ViewCell::new()
                .with_character('▘')
                .with_foreground(colours::WINDOWS),
            view_context,
        );
    }
}

pub fn window_x<F: Frame, C: ColModify>(view_context: ViewContext<C>, frame: &mut F) {
    for offset in Size::new_u16(3, 3).coord_iter_row_major() {
        frame.set_cell_relative(
            offset,
            0,
            ViewCell::new()
                .with_character(' ')
                .with_background(colours::WALL_TOP),
            view_context,
        );
    }
    frame.set_cell_relative(
        Coord { x: 1, y: 1 },
        0,
        ViewCell::new()
            .with_character(' ')
            .with_background(colours::WINDOWS),
        view_context,
    );
    frame.set_cell_relative(
        Coord { x: 0, y: 1 },
        0,
        ViewCell::new()
            .with_character('▌')
            .with_background(colours::WINDOWS)
            .with_foreground(colours::WALL_TOP),
        view_context,
    );
    frame.set_cell_relative(
        Coord { x: 2, y: 1 },
        0,
        ViewCell::new()
            .with_character('▌')
            .with_background(colours::WALL_TOP)
            .with_foreground(colours::WINDOWS),
        view_context,
    );
    frame.set_cell_relative(
        Coord { x: 0, y: 2 },
        0,
        ViewCell::new()
            .with_character('▝')
            .with_foreground(colours::WALL_FRONT),
        view_context,
    );
    frame.set_cell_relative(
        Coord { x: 2, y: 2 },
        0,
        ViewCell::new()
            .with_character('▘')
            .with_foreground(colours::WALL_FRONT),
        view_context,
    );
    frame.set_cell_relative(
        Coord { x: 1, y: 2 },
        0,
        ViewCell::new()
            .with_character('▀')
            .with_foreground(colours::WALL_FRONT),
        view_context,
    );
}

pub fn door_closed_y<F: Frame, C: ColModify>(view_context: ViewContext<C>, frame: &mut F) {
    for offset in Size::new_u16(3, 1).coord_iter_row_major() {
        frame.set_cell_relative(
            offset + Coord { x: 0, y: 1 },
            0,
            ViewCell::new()
                .with_character(' ')
                .with_background(colours::DOOR),
            view_context,
        );
        frame.set_cell_relative(
            offset + Coord { x: 0, y: 0 },
            0,
            ViewCell::new()
                .with_character('▄')
                .with_foreground(colours::DOOR_BORDER)
                .with_background(colours::FLOOR_BACKGROUND),
            view_context,
        );
        frame.set_cell_relative(
            offset + Coord { x: 0, y: 2 },
            0,
            ViewCell::new()
                .with_character('▄')
                .with_foreground(colours::FLOOR_BACKGROUND)
                .with_background(colours::DOOR_BORDER),
            view_context,
        );
    }
    frame.set_cell_relative(
        Coord { x: 0, y: 1 },
        0,
        ViewCell::new()
            .with_character('▌')
            .with_foreground(colours::DOOR_BORDER)
            .with_background(colours::DOOR),
        view_context,
    );
    frame.set_cell_relative(
        Coord { x: 2, y: 1 },
        0,
        ViewCell::new()
            .with_character('▌')
            .with_foreground(colours::DOOR)
            .with_background(colours::DOOR_BORDER),
        view_context,
    );
    frame.set_cell_relative(
        Coord { x: 1, y: 1 },
        0,
        ViewCell::new()
            .with_character('│')
            .with_foreground(colours::DOOR_BORDER)
            .with_bold(true),
        view_context,
    );
}

pub fn door_closed_x<F: Frame, C: ColModify>(view_context: ViewContext<C>, frame: &mut F) {
    for offset in Size::new_u16(1, 3).coord_iter_row_major() {
        frame.set_cell_relative(
            offset + Coord { x: 1, y: 0 },
            0,
            ViewCell::new()
                .with_character(' ')
                .with_background(colours::DOOR),
            view_context,
        );
        frame.set_cell_relative(
            offset + Coord { x: 0, y: 0 },
            0,
            ViewCell::new()
                .with_character(' ')
                .with_background(colours::FLOOR_BACKGROUND),
            view_context,
        );
        frame.set_cell_relative(
            offset + Coord { x: 0, y: 0 },
            0,
            ViewCell::new()
                .with_character('▌')
                .with_background(colours::DOOR_BORDER)
                .with_foreground(colours::FLOOR_BACKGROUND),
            view_context,
        );
        frame.set_cell_relative(
            offset + Coord { x: 2, y: 0 },
            0,
            ViewCell::new()
                .with_character('▌')
                .with_background(colours::FLOOR_BACKGROUND)
                .with_foreground(colours::DOOR_BORDER),
            view_context,
        );
    }
    frame.set_cell_relative(
        Coord { x: 1, y: 1 },
        0,
        ViewCell::new()
            .with_character('─')
            .with_foreground(colours::DOOR_BORDER)
            .with_bold(true),
        view_context,
    );
    frame.set_cell_relative(
        Coord { x: 1, y: 0 },
        0,
        ViewCell::new()
            .with_character('▀')
            .with_foreground(colours::DOOR_BORDER),
        view_context,
    );
    frame.set_cell_relative(
        Coord { x: 1, y: 2 },
        0,
        ViewCell::new()
            .with_character('▄')
            .with_foreground(colours::DOOR_BORDER),
        view_context,
    );
}

pub fn door_open_y<F: Frame, C: ColModify>(view_context: ViewContext<C>, frame: &mut F) {
    frame.set_cell_relative(
        Coord { x: 0, y: 1 },
        0,
        ViewCell::new()
            .with_character('▌')
            .with_foreground(colours::DOOR_BORDER),
        view_context,
    );
    frame.set_cell_relative(
        Coord { x: 2, y: 1 },
        0,
        ViewCell::new()
            .with_character('▐')
            .with_foreground(colours::DOOR_BORDER),
        view_context,
    );
    frame.set_cell_relative(
        Coord { x: 2, y: 0 },
        0,
        ViewCell::new()
            .with_character('▗')
            .with_foreground(colours::DOOR_BORDER),
        view_context,
    );
    frame.set_cell_relative(
        Coord { x: 0, y: 0 },
        0,
        ViewCell::new()
            .with_character('▖')
            .with_foreground(colours::DOOR_BORDER),
        view_context,
    );
    frame.set_cell_relative(
        Coord { x: 2, y: 2 },
        0,
        ViewCell::new()
            .with_character('▝')
            .with_foreground(colours::DOOR_BORDER),
        view_context,
    );
    frame.set_cell_relative(
        Coord { x: 0, y: 2 },
        0,
        ViewCell::new()
            .with_character('▘')
            .with_foreground(colours::DOOR_BORDER),
        view_context,
    );
}

pub fn door_open_x<F: Frame, C: ColModify>(view_context: ViewContext<C>, frame: &mut F) {
    frame.set_cell_relative(
        Coord { x: 2, y: 0 },
        0,
        ViewCell::new()
            .with_character('▘')
            .with_foreground(colours::DOOR_BORDER),
        view_context,
    );
    frame.set_cell_relative(
        Coord { x: 0, y: 0 },
        0,
        ViewCell::new()
            .with_character('▝')
            .with_foreground(colours::DOOR_BORDER),
        view_context,
    );
    frame.set_cell_relative(
        Coord { x: 2, y: 2 },
        0,
        ViewCell::new()
            .with_character('▖')
            .with_foreground(colours::DOOR_BORDER),
        view_context,
    );
    frame.set_cell_relative(
        Coord { x: 0, y: 2 },
        0,
        ViewCell::new()
            .with_character('▗')
            .with_foreground(colours::DOOR_BORDER),
        view_context,
    );
    frame.set_cell_relative(
        Coord { x: 1, y: 0 },
        0,
        ViewCell::new()
            .with_character('▀')
            .with_foreground(colours::DOOR_BORDER),
        view_context,
    );
    frame.set_cell_relative(
        Coord { x: 1, y: 2 },
        0,
        ViewCell::new()
            .with_character('▄')
            .with_foreground(colours::DOOR_BORDER),
        view_context,
    );
}

pub fn stairs<F: Frame, C: ColModify>(view_context: ViewContext<C>, frame: &mut F) {
    for offset in Size::new_u16(3, 3).coord_iter_row_major() {
        frame.set_cell_relative(
            offset,
            0,
            ViewCell::new()
                .with_character(' ')
                .with_background(colours::STAIRS_BACKGROUND),
            view_context,
        );
    }
    for offset in Size::new_u16(1, 3).coord_iter_row_major() {
        frame.set_cell_relative(
            offset,
            0,
            ViewCell::new()
                .with_character(' ')
                .with_background(colours::STAIRS_0),
            view_context,
        );
    }
    for offset in Size::new_u16(1, 2).coord_iter_row_major() {
        frame.set_cell_relative(
            offset + Coord { x: 1, y: 1 },
            0,
            ViewCell::new()
                .with_character(' ')
                .with_background(colours::STAIRS_1),
            view_context,
        );
    }
    frame.set_cell_relative(
        Coord { x: 2, y: 2 },
        0,
        ViewCell::new()
            .with_character(' ')
            .with_background(colours::STAIRS_2),
        view_context,
    );
}

pub fn zombie<F: Frame, C: ColModify>(
    entity: &ToRenderEntity,
    view_context: ViewContext<C>,
    frame: &mut F,
) {
    for offset in Size::new_u16(3, 3).coord_iter_row_major() {
        frame.set_cell_relative(
            offset,
            0,
            ViewCell::new()
                .with_character(' ')
                .with_background(colours::FLOOR_BACKGROUND),
            view_context,
        );
    }
    StringViewSingleLine::new(
        Style::new()
            .with_foreground(colours::ZOMBIE)
            .with_bold(true),
    )
    .view("Zmb", view_context, frame);
    StringViewSingleLine::new(
        Style::new()
            .with_foreground(colours::ZOMBIE)
            .with_bold(false),
    )
    .view(
        format!("♦{:02}", entity.armour.unwrap().value).as_str(),
        view_context.add_offset(Coord { x: 0, y: 1 }),
        frame,
    );
    StringViewSingleLine::new(
        Style::new()
            .with_foreground(colours::ZOMBIE)
            .with_bold(false),
    )
    .view(
        format!("♥{:02}", entity.hit_points.unwrap().current).as_str(),
        view_context.add_offset(Coord { x: 0, y: 2 }),
        frame,
    );
}

pub fn bullet<F: Frame, C: ColModify>(view_context: ViewContext<C>, frame: &mut F) {
    frame.set_cell_relative(
        Coord { x: 1, y: 1 },
        1,
        ViewCell::new()
            .with_character(' ')
            .with_background(colours::BULLET),
        view_context,
    );
}
