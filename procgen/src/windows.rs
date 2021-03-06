use crate::doors::DoorCell;
use crate::hull::HullCell;
use direction::CardinalDirections;
use grid_2d::{coord_2d::Axis, Coord, Grid};
use rand::seq::SliceRandom;
use rand::Rng;
use std::collections::{BTreeSet, HashSet};

type WindowCandidateId = usize;

#[derive(Debug)]
struct WindowCandidate {
    axis: Axis,
    top_left: Coord,
    length: u32,
    external: bool,
}

impl WindowCandidate {
    fn single_window_coord<R: Rng>(&self, rng: &mut R) -> Coord {
        let offset = rng.gen_range(0..(self.length as i32));
        let offset_coord = self.axis.new_coord(0, offset);
        self.top_left + offset_coord
    }

    fn next_coord(&self) -> Coord {
        self.top_left + self.axis.new_coord(0, self.length as i32)
    }
}

fn identify_window_candidates_in_axis(grid: &Grid<DoorCell>, axis: Axis) -> Vec<WindowCandidate> {
    let mut window_candidates: Vec<WindowCandidate> = Vec::new();
    for i in 0..grid.size().get(axis) {
        for j in 0..grid.size().get(axis.other()) {
            let coord = axis.new_coord(i as i32, j as i32);
            let &cell = grid.get_checked(coord);
            if cell == DoorCell::Wall {
                let l = coord + axis.new_coord(-1, 0);
                let r = coord + axis.new_coord(1, 0);
                let lc = grid.get(l).cloned().unwrap_or(DoorCell::Space);
                let rc = grid.get(r).cloned().unwrap_or(DoorCell::Space);
                if lc != DoorCell::Floor && lc != DoorCell::Space {
                    continue;
                }
                if rc != DoorCell::Floor && rc != DoorCell::Space {
                    continue;
                }
                let external = lc == DoorCell::Space || rc == DoorCell::Space;
                let mut updated_current = false;
                if let Some(last_candidate) = window_candidates.last_mut() {
                    if last_candidate.external == external && last_candidate.next_coord() == coord {
                        last_candidate.length += 1;
                        updated_current = true;
                    }
                }
                if !updated_current {
                    window_candidates.push(WindowCandidate {
                        axis,
                        top_left: coord,
                        length: 1,
                        external,
                    });
                }
            }
        }
    }
    window_candidates
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum WindowCell {
    Wall,
    Floor,
    Space,
    Door(Axis),
    Window(Axis),
    Stairs,
    Spawn,
}

pub fn add_windows<R: Rng>(grid: &Grid<DoorCell>, rng: &mut R) -> Grid<WindowCell> {
    let mut out_grid = grid.map_ref(|cell| match cell {
        DoorCell::Floor => WindowCell::Floor,
        DoorCell::Wall => WindowCell::Wall,
        DoorCell::Space => WindowCell::Space,
        DoorCell::Door(axis) => WindowCell::Door(*axis),
        DoorCell::Stairs => WindowCell::Stairs,
        DoorCell::Spawn => WindowCell::Spawn,
    });
    let window_candidates_x = identify_window_candidates_in_axis(grid, Axis::X);
    let window_candidates_y = identify_window_candidates_in_axis(grid, Axis::Y);
    let mut window_candidates_internal = window_candidates_x
        .iter()
        .chain(window_candidates_y.iter())
        .filter(|wc| !wc.external)
        .collect::<Vec<_>>();
    let mut window_candidates_external = window_candidates_x
        .iter()
        .chain(window_candidates_y.iter())
        .filter(|wc| wc.external)
        .collect::<Vec<_>>();
    window_candidates_internal.shuffle(rng);
    window_candidates_external.shuffle(rng);
    let num_window_candidates_internal = window_candidates_internal.len() / 4;
    let num_window_candidates_external = (window_candidates_external.len() * 2) / 3;
    for window_candidate in window_candidates_internal
        .iter()
        .take(num_window_candidates_internal)
    {
        *out_grid.get_checked_mut(window_candidate.single_window_coord(rng)) =
            WindowCell::Window(window_candidate.axis);
    }
    for window_candidate in window_candidates_external
        .iter()
        .take(num_window_candidates_external)
    {
        *out_grid.get_checked_mut(window_candidate.single_window_coord(rng)) =
            WindowCell::Window(window_candidate.axis);
        if rng.gen_range(0..2) == 0 {
            *out_grid.get_checked_mut(window_candidate.single_window_coord(rng)) =
                WindowCell::Window(window_candidate.axis);
        }
    }

    out_grid
}
