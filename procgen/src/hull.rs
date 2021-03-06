use direction::Direction;
use grid_2d::{Coord, Grid, Size};
use rand::Rng;
use std::num::NonZeroU32;
use wfc::{overlapping::OverlappingPatterns, retry, wrap, ForbidNothing, RunOwn};

#[rustfmt::skip]
const INPUT: &[&str] = &[
"................",
"...###..........",
"..##.##.........",
"..#...#...####..",
"..#...#...#..#..",
"..#...#...#..#..",
"..#...#...#..#..",
"..#...#...#..#..",
"..#...#...#..#..",
"..#...#####..#..",
"..#..........#..",
"..#..........#..",
"..#....####..#..",
"..######..####..",
"................",

];

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
enum GenerationCell {
    Closed,
    Open,
}

fn input_grid_from_strs(input: &[&str]) -> Grid<GenerationCell> {
    let width = input[0].len();
    let height = input.len();
    let size = Size::new(width as u32, height as u32);
    let mut grid = Grid::new_clone(size, GenerationCell::Open);
    for (y, row) in input.iter().enumerate() {
        for (x, ch) in row.chars().enumerate() {
            let coord = Coord::new(x as i32, y as i32);
            let cell = match ch {
                '.' => GenerationCell::Open,
                '#' => GenerationCell::Closed,
                ch => panic!("unexpected char: {}", ch),
            };
            *grid.get_checked_mut(coord) = cell;
        }
    }
    grid
}

fn wfc_map<R: Rng>(
    input_grid: Grid<GenerationCell>,
    output_size: Size,
    pattern_size: NonZeroU32,
    rng: &mut R,
) -> Grid<GenerationCell> {
    let mut output_grid = Grid::new_clone(output_size, GenerationCell::Open);
    let overlapping_patterns = OverlappingPatterns::new_all_orientations(input_grid, pattern_size);
    let global_stats = overlapping_patterns.global_stats();
    let run = RunOwn::new_wrap_forbid(output_size, &global_stats, wrap::WrapXY, ForbidNothing, rng);
    let wave = run.collapse_retrying(retry::Forever, rng);
    for (coord, wave_cell) in wave.grid().enumerate() {
        let pattern_id = wave_cell
            .chosen_pattern_id()
            .expect("unexpected contradiction");
        let cell = overlapping_patterns.pattern_top_left_value(pattern_id);
        *output_grid.get_checked_mut(coord) = *cell;
    }
    output_grid
}

fn keep_largest_enclosed_area(grid: &Grid<GenerationCell>) -> Grid<GenerationCell> {
    let mut visited_ids: Grid<Option<usize>> = Grid::new_clone(grid.size(), None);
    let mut flood_fill_buffer = Vec::new();
    let mut current_id = 0usize;
    let mut counts_by_id = Vec::new();
    for (coord, cell) in grid.enumerate() {
        if let GenerationCell::Open = cell {
            if visited_ids.get_checked(coord).is_none() {
                flood_fill_buffer.push(coord);
                *visited_ids.get_checked_mut(coord) = Some(current_id);
                let mut count = 0usize;
                while let Some(coord) = flood_fill_buffer.pop() {
                    count += 1;
                    for direction in Direction::all() {
                        let next_coord = coord + direction.coord();
                        match grid.get(next_coord) {
                            None | Some(GenerationCell::Closed) => continue,
                            Some(GenerationCell::Open) => (),
                        }
                        let maybe_visited_id = visited_ids.get_checked_mut(next_coord);
                        if maybe_visited_id.is_none() {
                            *maybe_visited_id = Some(current_id);
                            flood_fill_buffer.push(next_coord);
                        }
                    }
                }
                counts_by_id.push(count);
                current_id += 1;
            }
        }
    }
    let (id_of_largest_area, _count) = counts_by_id
        .iter()
        .enumerate()
        .max_by_key(|&(_, count)| count)
        .expect("found no enclosed areas");
    let grid_keeping_largest_enclosed_area =
        Grid::new_grid_map_ref(&visited_ids, |&maybe_id| match maybe_id {
            Some(id) => {
                if id == id_of_largest_area {
                    GenerationCell::Open
                } else {
                    GenerationCell::Closed
                }
            }
            None => GenerationCell::Closed,
        });
    grid_keeping_largest_enclosed_area
}

fn wrap_in_closed_area(grid: &Grid<GenerationCell>) -> Grid<GenerationCell> {
    Grid::new_fn(grid.size() + Size::new(2, 2), |coord| {
        if let Some(cell) = grid.get(coord - Coord::new(1, 1)) {
            *cell
        } else {
            GenerationCell::Closed
        }
    })
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum HullCell {
    Wall,
    Floor,
    Space,
}

fn strip_walls_from_outside(grid: &Grid<GenerationCell>) -> Grid<HullCell> {
    Grid::new_grid_map_ref_with_coord(grid, |coord, cell| match cell {
        GenerationCell::Open => HullCell::Floor,
        GenerationCell::Closed => {
            for direction in Direction::all() {
                let neighbour_coord = coord + direction.coord();
                if let Some(GenerationCell::Open) = grid.get(neighbour_coord) {
                    return HullCell::Wall;
                }
            }
            HullCell::Space
        }
    })
}

fn surround_by_space(grid: &Grid<HullCell>) -> Grid<HullCell> {
    Grid::new_fn(grid.size(), |coord| {
        if let Some(&cell) = grid.get(coord) {
            cell
        } else {
            HullCell::Space
        }
    })
}

fn generate_hull_internal<R: Rng>(
    input_grid: Grid<GenerationCell>,
    output_size: Size,
    pattern_size: NonZeroU32,
    rng: &mut R,
) -> Grid<HullCell> {
    let output_grid = wfc_map(input_grid, output_size - Size::new(2, 2), pattern_size, rng);
    let output_grid = keep_largest_enclosed_area(&output_grid);
    let output_grid = wrap_in_closed_area(&output_grid);
    let output_grid = strip_walls_from_outside(&output_grid);
    let output_grid = surround_by_space(&output_grid);
    output_grid
}

fn try_generate_hull<R: Rng>(output_size: Size, rng: &mut R) -> Grid<HullCell> {
    let input_grid = input_grid_from_strs(INPUT);
    let pattern_size = NonZeroU32::new(4).unwrap();
    generate_hull_internal(input_grid, output_size, pattern_size, rng)
}

fn hull_bounding_box(hull: &Grid<HullCell>) -> Size {
    let mut min = hull.size();
    let mut max = Size::new(0, 0);
    for (coord, &cell) in hull.enumerate() {
        if cell != HullCell::Space {
            min.set_width_in_place(min.width().min(coord.x as u32));
            min.set_height_in_place(min.height().min(coord.y as u32));
            max.set_width_in_place(max.width().max(coord.x as u32));
            max.set_height_in_place(max.height().max(coord.y as u32));
        }
    }
    max - min
}

pub fn generate_hull<R: Rng>(output_size: Size, rng: &mut R) -> Grid<HullCell> {
    loop {
        let hull = try_generate_hull(output_size, rng);
        let floor_count = hull.iter().filter(|&&c| c == HullCell::Floor).count();
        if floor_count < (output_size.count() * 1) / 2 {
            continue;
        }
        let bounding_box = hull_bounding_box(&hull);
        let delta_x = output_size.width() - bounding_box.width();
        let delta_y = output_size.height() - bounding_box.height();
        if delta_x + delta_y > 8 {
            continue;
        }
        return hull;
    }
}
