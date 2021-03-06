use crate::hull::HullCell;
use direction::CardinalDirections;
use grid_2d::{coord_2d::Axis, Coord, Grid};
use rand::seq::SliceRandom;
use rand::Rng;
use std::collections::{BTreeSet, HashSet};

type RoomId = usize;

#[derive(Clone, Copy)]
struct FloorCell {
    room_id: RoomId,
}

fn classify_floor_cells_into_rooms(grid: &Grid<HullCell>) -> Grid<Option<FloorCell>> {
    let mut floor_grid = grid.map_ref(|_| None);
    let mut next_room_id = 0;
    let mut seen = HashSet::new();
    for (coord, &cell) in grid.enumerate() {
        if cell == HullCell::Floor {
            if seen.insert(coord) {
                let mut to_visit = vec![coord];
                while let Some(coord) = to_visit.pop() {
                    *floor_grid.get_checked_mut(coord) = Some(FloorCell {
                        room_id: next_room_id,
                    });
                    for direction in CardinalDirections {
                        let nei_coord = coord + direction.coord();
                        if let Some(HullCell::Floor) = grid.get(nei_coord) {
                            if seen.insert(nei_coord) {
                                to_visit.push(nei_coord);
                            }
                        }
                    }
                }
                next_room_id += 1;
            }
        }
    }
    floor_grid
}

type DoorCandidateId = usize;

#[derive(Debug)]
struct DoorCandidate {
    axis: Axis,
    top_left: Coord,
    length: u32,
    left_room_id: RoomId,
    right_room_id: RoomId,
}

impl DoorCandidate {
    fn door_coord<R: Rng>(&self, rng: &mut R) -> Coord {
        let offset = rng.gen_range(0..(self.length as i32));
        let offset_coord = self.axis.new_coord(0, offset);
        self.top_left + offset_coord
    }

    fn next_coord(&self) -> Coord {
        self.top_left + self.axis.new_coord(0, self.length as i32)
    }
}

fn identify_door_candidates_in_axis(
    hull: &Grid<HullCell>,
    floor: &Grid<Option<FloorCell>>,
    axis: Axis,
) -> Vec<DoorCandidate> {
    let mut door_candidates: Vec<DoorCandidate> = Vec::new();
    for i in 0..hull.size().get(axis) {
        for j in 0..hull.size().get(axis.other()) {
            let coord = axis.new_coord(i as i32, j as i32);
            let &hull_cell = hull.get_checked(coord);
            if hull_cell == HullCell::Wall {
                let l = coord + axis.new_coord(-1, 0);
                let r = coord + axis.new_coord(1, 0);
                if let Some(Some(lc)) = floor.get(l) {
                    if let Some(Some(rc)) = floor.get(r) {
                        let mut updated_current = false;
                        if lc.room_id != rc.room_id {
                            if let Some(last_candidate) = door_candidates.last_mut() {
                                if last_candidate.left_room_id == lc.room_id
                                    && last_candidate.right_room_id == rc.room_id
                                    && last_candidate.next_coord() == coord
                                {
                                    last_candidate.length += 1;
                                    updated_current = true;
                                }
                            }
                        }
                        if !updated_current {
                            door_candidates.push(DoorCandidate {
                                axis,
                                top_left: coord,
                                length: 1,
                                left_room_id: lc.room_id,
                                right_room_id: rc.room_id,
                            });
                        }
                    }
                }
            }
        }
    }
    door_candidates
}

struct RoomEdge {
    to_room_id: RoomId,
    via: DoorCandidateId,
}

struct RoomNode {
    edges: Vec<RoomEdge>,
}

struct RoomGraph {
    door_candidates: Vec<DoorCandidate>,
    nodes: Vec<RoomNode>,
}

impl RoomGraph {
    fn new(door_candidates: Vec<DoorCandidate>) -> Self {
        let num_rooms = door_candidates
            .iter()
            .map(|dc| dc.left_room_id.max(dc.right_room_id))
            .max()
            .unwrap()
            + 1;
        let mut nodes = (0..num_rooms)
            .map(|_| RoomNode { edges: Vec::new() })
            .collect::<Vec<_>>();
        for (door_candidate_id, door_candidate) in door_candidates.iter().enumerate() {
            nodes[door_candidate.left_room_id].edges.push(RoomEdge {
                to_room_id: door_candidate.right_room_id,
                via: door_candidate_id,
            });
            nodes[door_candidate.right_room_id].edges.push(RoomEdge {
                to_room_id: door_candidate.left_room_id,
                via: door_candidate_id,
            });
        }
        Self {
            door_candidates,
            nodes,
        }
    }

    fn random_door_candidate_minimum_spanning_tree<R: Rng>(
        &self,
        rng: &mut R,
    ) -> BTreeSet<DoorCandidateId> {
        let mut minimum_spanning_tree = BTreeSet::new();
        let mut visited_room_ids = HashSet::new();
        let mut to_visit = vec![rng.gen_range(0..self.door_candidates.len())];
        while !to_visit.is_empty() {
            let door_candidate_id = to_visit.swap_remove(rng.gen_range(0..to_visit.len()));
            let door_candidate = &self.door_candidates[door_candidate_id];
            let new_left = visited_room_ids.insert(door_candidate.left_room_id);
            let new_right = visited_room_ids.insert(door_candidate.right_room_id);
            if !(new_left || new_right) {
                // every step must visit a previously unvisited room
                continue;
            }
            minimum_spanning_tree.insert(door_candidate_id);
            for edge in self.nodes[door_candidate.left_room_id]
                .edges
                .iter()
                .chain(self.nodes[door_candidate.right_room_id].edges.iter())
            {
                if !visited_room_ids.contains(&edge.to_room_id) {
                    to_visit.push(edge.via);
                }
            }
        }
        minimum_spanning_tree
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum DoorCell {
    Wall,
    Floor,
    Space,
    Door(Axis),
    Stairs,
    Spawn,
}

fn choose_stairs_coord<R: Rng>(grid: &Grid<DoorCell>, rng: &mut R) -> Option<Coord> {
    let mut candidates = grid
        .enumerate()
        .filter(|(coord, &cell)| {
            cell == DoorCell::Floor
                && CardinalDirections
                    .into_iter()
                    .map(|d| coord + d.coord())
                    .all(|coord| {
                        grid.get(coord)
                            .map(|&cell| cell == DoorCell::Floor)
                            .unwrap_or(false)
                    })
        })
        .map(|(coord, _)| coord)
        .collect::<Vec<_>>();
    candidates.shuffle(rng);
    candidates.pop()
}

fn choose_spawn_coord<R: Rng>(
    grid: &Grid<DoorCell>,
    floor: &Grid<Option<FloorCell>>,
    stairs_room: RoomId,
    stairs_coord: Coord,
    rng: &mut R,
) -> Option<Coord> {
    let mut candidates = grid
        .enumerate()
        .filter(|(coord, &cell)| {
            cell == DoorCell::Floor
                && floor.get_checked(*coord).unwrap().room_id != stairs_room
                && CardinalDirections
                    .into_iter()
                    .map(|d| coord + d.coord())
                    .all(|coord| {
                        grid.get(coord)
                            .map(|&cell| cell == DoorCell::Floor)
                            .unwrap_or(false)
                    })
        })
        .map(|(coord, _)| coord)
        .collect::<Vec<_>>();
    candidates.sort_by_key(|coord| (coord - stairs_coord).magnitude2());
    candidates.reverse();
    if candidates.is_empty() {
        None
    } else {
        candidates[0..(candidates.len() / 4).max(1)]
            .choose(rng)
            .cloned()
    }
}

pub fn add_doors<R: Rng>(hull: &Grid<HullCell>, rng: &mut R) -> Option<Grid<DoorCell>> {
    let floor = classify_floor_cells_into_rooms(hull);
    let door_candidates_x = identify_door_candidates_in_axis(hull, &floor, Axis::X);
    let door_candidates_y = identify_door_candidates_in_axis(hull, &floor, Axis::Y);
    let door_candidates = door_candidates_x
        .into_iter()
        .chain(door_candidates_y.into_iter())
        .collect::<Vec<_>>();
    let room_graph = RoomGraph::new(door_candidates);
    let mst = room_graph.random_door_candidate_minimum_spanning_tree(rng);
    let mut door_grid = hull.map_ref(|cell| match cell {
        HullCell::Floor => DoorCell::Floor,
        HullCell::Wall => DoorCell::Wall,
        HullCell::Space => DoorCell::Space,
    });
    let mut extra_door_candidates = (0..room_graph.door_candidates.len())
        .filter(|door_candidate_id| !mst.contains(&door_candidate_id))
        .collect::<Vec<_>>();
    extra_door_candidates.shuffle(rng);
    let num_extra_door_candidates = (extra_door_candidates.len() + 1) / 2;
    for door_candidate_id in mst.into_iter().chain(
        extra_door_candidates
            .into_iter()
            .take(num_extra_door_candidates),
    ) {
        let door_candidate = &room_graph.door_candidates[door_candidate_id];
        *door_grid.get_checked_mut(door_candidate.door_coord(rng)) =
            DoorCell::Door(door_candidate.axis);
    }
    let stairs_coord = choose_stairs_coord(&door_grid, rng)?;
    *door_grid.get_checked_mut(stairs_coord) = DoorCell::Stairs;
    let spawn_coord = choose_spawn_coord(
        &door_grid,
        &floor,
        floor.get_checked(stairs_coord).unwrap().room_id,
        stairs_coord,
        rng,
    )?;
    *door_grid.get_checked_mut(spawn_coord) = DoorCell::Spawn;
    Some(door_grid)
}
