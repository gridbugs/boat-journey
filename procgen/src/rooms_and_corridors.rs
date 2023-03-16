use coord_2d::{Axis, Coord, Size};
use direction::CardinalDirection;
use grid_2d::Grid;
use rand::{seq::SliceRandom, Rng};
use std::collections::HashSet;

// Will be used as cells in grids representing simple maps of levels during terrain generation
#[derive(Clone, Copy, PartialEq, Eq)]
enum FloorOrWall {
    Floor,
    Wall,
}

// An axis-aligned rectangle
#[derive(Clone, Copy)]
struct Rect {
    top_left: Coord,
    size: Size,
}

impl Rect {
    // Randomly generate a rectangle
    fn choose<R: Rng>(bounds: Size, min_size: Size, max_size: Size, rng: &mut R) -> Self {
        let width = rng.gen_range(min_size.width()..max_size.width());
        let height = rng.gen_range(min_size.height()..max_size.height());
        let size = Size::new(width, height);
        let top_left_bounds = bounds - size;
        let left = rng.gen_range(0..top_left_bounds.width());
        let top = rng.gen_range(0..top_left_bounds.height());
        let top_left = Coord::new(left as i32, top as i32);
        Self { top_left, size }
    }

    // Returns an iterator over all the coordinates in the rectangle
    fn coords(&self) -> impl '_ + Iterator<Item = Coord> {
        self.size.coord_iter_row_major().map(|c| c + self.top_left)
    }

    // Returns true iff the given coordinate is on the edge of the rectangle
    fn is_edge(&self, coord: Coord) -> bool {
        self.size.is_on_edge(coord - self.top_left)
    }

    // Returns an iterator over the edge coordinates of the rectangle
    fn edge_coords(&self) -> impl '_ + Iterator<Item = Coord> {
        self.size.edge_iter().map(|c| self.top_left + c)
    }

    // Returns an iterator over the internal (non-edge) coordinates of the rectangle
    fn internal_coords(&self) -> impl '_ + Iterator<Item = Coord> {
        self.coords().filter(|&c| !self.is_edge(c))
    }

    // Returns the coordinate of the centre of the rectangle
    fn centre(&self) -> Coord {
        self.top_left + (self.size / 2)
    }
}

// Represents a room during terrain generation
#[derive(Clone, Copy)]
struct Room {
    // The edge of the rectangle will be the walls surrounding the room, and the inside of the
    // rectangle will be the floor of the room.
    rect: Rect,
}

impl Room {
    // Returns true iff any cell of the room corresponds to a floor cell in the given map
    fn overlaps_with_floor(&self, map: &Grid<FloorOrWall>) -> bool {
        self.rect
            .coords()
            .any(|coord| *map.get_checked(coord) == FloorOrWall::Floor)
    }

    // Updates the given map, setting each cell corresponding to the floor of this room to be a
    // floor cell
    fn add_floor_to_map(&self, map: &mut Grid<FloorOrWall>) {
        for coord in self.rect.internal_coords() {
            *map.get_checked_mut(coord) = FloorOrWall::Floor;
        }
    }
}

// Checks whether a given cell of a map has a floor either side of it in the given axis, and a
// wall either side of it in the other axis. (An Axis is defined as `enum Axis { X, Y }`.)
// This is used to check whether a cell is suitable to contain a door.
fn is_cell_in_corridor_axis(map: &Grid<FloorOrWall>, coord: Coord, axis: Axis) -> bool {
    use FloorOrWall::*;
    let axis_delta = Coord::new_axis(1, 0, axis);
    let other_axis_delta = Coord::new_axis(0, 1, axis);
    let floor_in_axis = *map.get_checked(coord + axis_delta) == Floor
        && *map.get_checked(coord - axis_delta) == Floor;
    let wall_in_other_axis = *map.get_checked(coord + other_axis_delta) == Wall
        && *map.get_checked(coord - other_axis_delta) == Wall;
    floor_in_axis && wall_in_other_axis
}

// Checks whether a given cell of a map has a floor either side of it in some axis, and a wall
// either side of it in the other axis.
// This is used to check whether a cell is suitable to contain a door.
fn is_cell_in_corridor(map: &Grid<FloorOrWall>, coord: Coord) -> bool {
    is_cell_in_corridor_axis(map, coord, Axis::X) || is_cell_in_corridor_axis(map, coord, Axis::Y)
}

// Checks whether a cell has any neighbours which are floors
fn has_floor_neighbour(map: &Grid<FloorOrWall>, coord: Coord) -> bool {
    CardinalDirection::all().any(|d| *map.get_checked(coord + d.coord()) == FloorOrWall::Floor)
}

// Returns a vec of coordinates that define an L-shaped corridor from start to end (in order). The
// corridor stops if it encounters a cell adjacent to a floor cell according to the given map. The
// first axis that is traversed in the L-shaped corridor will be the given axis.
fn l_shaped_corridor_with_first_axis(
    start: Coord,
    end: Coord,
    map: &Grid<FloorOrWall>,
    first_axis: Axis,
) -> Vec<Coord> {
    let mut ret = Vec::new();
    let delta = end - start;
    let step = Coord::new_axis(delta.get(first_axis).signum(), 0, first_axis);
    // Skip the start coordinate so multiple corridors can start from the same coord
    let mut current = start + step;
    while current.get(first_axis) != end.get(first_axis) {
        ret.push(current);
        if has_floor_neighbour(map, current) {
            // stop when we get adjacent to a floor cell
            return ret;
        }
        current += step;
    }
    let step = Coord::new_axis(0, delta.get(first_axis.other()).signum(), first_axis);
    while current != end {
        ret.push(current);
        if has_floor_neighbour(map, current) {
            // stop when we get adjacent to a floor cell
            return ret;
        }
        current += step;
    }
    ret
}

// Returns a vec of coordinates that define an L-shaped corridor from start to end (in order). The
// corridor stops if it encounters a cell adjacent to a floor cell according to the given map. The
// first axis that is traversed in the L-shaped corridor is chosen at random.
fn l_shaped_corridor<R: Rng>(
    start: Coord,
    end: Coord,
    map: &Grid<FloorOrWall>,
    rng: &mut R,
) -> Vec<Coord> {
    let axis = if rng.gen() { Axis::X } else { Axis::Y };
    l_shaped_corridor_with_first_axis(start, end, map, axis)
}

// Data structure representing the state of the room-placement algorithm
struct RoomPlacement {
    // A list of rooms that have been placed
    rooms: Vec<Room>,
    // A set of all coordinates that are the edge of a room
    edge_coords: HashSet<Coord>,
    // List of cells that would be suitable to contain doors
    door_candidates: Vec<Coord>,
    // Tracks whether there is a wall or floor at each location
    map: Grid<FloorOrWall>,
}

impl RoomPlacement {
    fn new(size: Size) -> Self {
        Self {
            rooms: Vec::new(),
            edge_coords: HashSet::new(),
            door_candidates: Vec::new(),
            map: Grid::new_copy(size, FloorOrWall::Wall),
        }
    }

    // Adds a new room unless it overlaps with the floor
    fn try_add_room<R: Rng>(&mut self, new_room: Room, rng: &mut R) {
        // Don't add the room if it overlaps with the floor
        if new_room.overlaps_with_floor(&self.map) {
            return;
        }
        // Add the room's wall to the collection of edge coords
        self.edge_coords.extend(new_room.rect.edge_coords());
        // Randomly choose two rooms to connect the new room to
        for &existing_room in self.rooms.choose_multiple(rng, 2) {
            // List the coordinates of an L-shaped corridor between the centres of the new room and
            // the chosen exsiting room
            let corridor = l_shaped_corridor(
                new_room.rect.centre(),
                existing_room.rect.centre(),
                &self.map,
                rng,
            );
            // Carve out the corridor from the map
            for &coord in &corridor {
                *self.map.get_checked_mut(coord) = FloorOrWall::Floor;
            }
            // Update the list of door candidates along this corridor
            let mut door_candidate = None;
            for &coord in &corridor {
                if self.edge_coords.contains(&coord) && is_cell_in_corridor(&self.map, coord) {
                    door_candidate = Some(coord);
                } else if let Some(coord) = door_candidate.take() {
                    // The candidate is stored in door_candidate (an Option<Coord>) until a
                    // non-candidate cell is found, at which point the currently-stored candidate
                    // is added to the list of door candidates. This prevents multiple consecutive
                    // door candidates being added, which could result in several doors in a row
                    // which is undesired.
                    self.door_candidates.push(coord);
                }
            }
            if let Some(coord) = door_candidate {
                self.door_candidates.push(coord);
            }
        }
        new_room.add_floor_to_map(&mut self.map);
        self.rooms.push(new_room);
    }
}

// A cell of the RoomsAndCorridorsLevel map
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum RoomsAndCorridorsCell {
    Floor,
    Wall,
    Door,
}

// Represents a level made up of rooms and corridors
pub struct RoomsAndCorridorsLevel {
    // Whether each cell is a floor or wall
    pub map: Grid<RoomsAndCorridorsCell>,
    // Location where the player will start
    pub player_spawn: Coord,
    // Player's destination (e.g. stairs to next level)
    pub destination: Coord,
    pub other_room_centres: Vec<Coord>,
}

impl RoomsAndCorridorsLevel {
    // Randomly generates a level made up of rooms and corridors
    pub fn generate<R: Rng>(size: Size, rng: &mut R) -> Self {
        const NUM_ROOM_ATTEMPTS: usize = 50;
        const MIN_ROOM_SIZE: Size = Size::new_u16(5, 5);
        const MAX_ROOM_SIZE: Size = Size::new_u16(11, 9);
        let mut room_placement = RoomPlacement::new(size);
        // Add all the rooms and corridors
        for _ in 0..NUM_ROOM_ATTEMPTS {
            let new_room = Room {
                rect: Rect::choose(size, MIN_ROOM_SIZE, MAX_ROOM_SIZE, rng),
            };
            room_placement.try_add_room(new_room, rng);
        }
        // Create the map made of `RoomsAndCorridorsCell`s
        let mut map = Grid::new_grid_map(room_placement.map, |floor_or_wall| match floor_or_wall {
            FloorOrWall::Floor => RoomsAndCorridorsCell::Floor,
            FloorOrWall::Wall => RoomsAndCorridorsCell::Wall,
        });
        // Add doors
        for door_candidate_coord in room_placement.door_candidates {
            // Each door candidate has a 50% chance to become a door
            if rng.gen::<bool>() {
                *map.get_checked_mut(door_candidate_coord) = RoomsAndCorridorsCell::Door;
            }
        }
        // The player will start in the centre of a randomly-chosen room
        let player_spawn = room_placement.rooms.choose(rng).unwrap().rect.centre();

        // The destination will be in centre of the room furthest from the player spawn
        let destination = room_placement
            .rooms
            .iter()
            .max_by_key(|room| (room.rect.centre() - player_spawn).magnitude2())
            .unwrap()
            .rect
            .centre();

        let other_room_centres = room_placement
            .rooms
            .iter()
            .cloned()
            .filter(|c| c.rect.centre() != player_spawn && c.rect.centre() != destination)
            .map(|c| c.rect.centre())
            .collect::<Vec<_>>();
        Self {
            map,
            player_spawn,
            destination,
            other_room_centres,
        }
    }
}
