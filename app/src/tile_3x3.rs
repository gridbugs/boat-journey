use crate::colours;
use chargrid::render::{
    grid_2d::coord_2d::Axis, ColModify, Coord, Frame, Size, Style, View, ViewCell, ViewContext,
};
use chargrid::text::StringViewSingleLine;
use orbital_decay_game::{Game, Tile, ToRenderEntity};

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
            if game.contains_wall(below) && (!game.visibility_grid().is_coord_never_visible(below))
            {
                wall_top(view_context, frame);
            } else {
                wall_front(view_context, frame);
            }
        }
        Tile::Floor => floor(view_context, frame),
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
