use gridbugs::{
    coord_2d::{Coord, Size},
    direction::{CardinalDirection, Direction},
    grid_2d::Grid,
    line_2d,
    perlin2::Perlin2,
};
use rand::{seq::SliceRandom, Rng};
use std::{
    cmp::Ordering,
    collections::{BinaryHeap, HashMap, HashSet, VecDeque},
};
use vector::{Cartesian, Radial, Radians};

mod rooms_and_corridors;

pub struct Spec {
    pub size: Size,
    pub num_graves: u32,
}

pub struct LandCell {
    pub height: f64,
}

pub struct Land {
    pub cells: Grid<LandCell>,
    pub height_diff: f64,
}

#[derive(Debug, PartialEq)]
struct SearchCell {
    cost: f64,
    heuristic: f64,
    coord: Coord,
}

impl PartialOrd for SearchCell {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        other.heuristic.partial_cmp(&self.heuristic)
    }
}

impl Eq for SearchCell {}

impl Ord for SearchCell {
    fn cmp(&self, other: &Self) -> Ordering {
        if self.heuristic < other.heuristic {
            Ordering::Greater
        } else if self.heuristic > other.heuristic {
            Ordering::Less
        } else {
            Ordering::Equal
        }
    }
}

impl Land {
    pub fn base_height_on_row(&self, row: usize) -> f64 {
        ((self.cells.width() as usize - row - 1) as f64 * self.height_diff)
            / self.cells.width() as f64
    }
    pub fn get_height(&self, coord: Coord) -> Option<f64> {
        self.cells.get(coord).map(|land_cell| {
            let mut h = self.base_height_on_row(coord.x as usize) + land_cell.height;
            if coord.y < 20 || coord.y > self.cells.height() as i32 - 20 {
                h += 40.;
            }
            h
        })
    }
    pub fn plot_river(&self) -> Vec<Coord> {
        let mut rows = (0..(self.cells.height() as usize)).collect::<Vec<_>>();
        rows.sort_by_key(|&i| {
            let coord = Coord::new(0, i as i32);
            let h = (self.get_height(coord).unwrap() * 1000.0) as i64;
            h
        });
        let mut best_cost = f64::MAX;
        let mut best = Vec::new();
        for i in rows.into_iter().take(4) {
            let (path, cost) = self.plot_river_from_row(i);
            if cost < best_cost {
                best_cost = cost;
                best = path;
            }
        }
        best
    }
    pub fn plot_river_from_row(&self, start_row: usize) -> (Vec<Coord>, f64) {
        let start_coord = Coord::new(0, start_row as i32);
        let start_cost = self.get_height(start_coord).unwrap();
        let start = SearchCell {
            coord: start_coord,
            cost: start_cost,
            heuristic: self.cells.width() as f64 + start_cost,
        };
        let mut seen: HashMap<Coord, f64> = HashMap::new();
        seen.insert(start.coord, start.cost);
        let mut pq: BinaryHeap<SearchCell> = BinaryHeap::new();
        pq.push(start);
        let mut chain: HashMap<Coord, Coord> = HashMap::new();
        let mut end = None;
        while let Some(cell) = pq.pop() {
            if cell.coord.x as u32 == self.cells.width() - 1 {
                end = Some(cell);
                break;
            }
            for d in Direction::all() {
                let neighbour_coord = cell.coord + d.coord();
                if let Some(neighbour_height) = self.get_height(neighbour_coord) {
                    let mut neighbour_cost = cell.cost + neighbour_height.powf(2.);
                    let top_bottom_barrier_width = 5;
                    let top_bottom_barrier_cost = 1000.;
                    if neighbour_coord.y < top_bottom_barrier_width
                        || neighbour_coord.y
                            >= (self.cells.height() as i32 - top_bottom_barrier_width)
                    {
                        neighbour_cost += top_bottom_barrier_cost;
                    }
                    if let Some(best_existing_neighbour_cost) = seen.get(&neighbour_coord) {
                        if *best_existing_neighbour_cost <= neighbour_cost {
                            continue;
                        }
                    }
                    seen.insert(neighbour_coord, neighbour_cost);
                    pq.push(SearchCell {
                        coord: neighbour_coord,
                        cost: neighbour_cost,
                        heuristic: neighbour_cost
                            + (self.cells.width() as f64 - neighbour_coord.x as f64),
                    });
                    chain.insert(neighbour_coord, cell.coord);
                }
            }
        }
        let end = end.unwrap();
        let mut current = end.coord;
        let mut sequence = vec![current];
        while let Some(coord) = chain.remove(&current) {
            current = coord;
            sequence.push(coord);
        }
        sequence.reverse();
        (sequence, end.cost)
    }
}

fn validate_river(river: &[Coord]) -> bool {
    let _ = river;
    true
    /*
    let n = 10;
    'outer: for w in river.windows((2 * n) + 1) {
        let delta_0 = w[1] - w[0];
        for i in 1..n {
            let delta = w[i + 1] - w[i];
            if delta != delta_0 {
                continue 'outer;
            }
        }
        let delta_n = w[n + 1] - w[n];
        for i in (n + 1)..(2 * n) {
            let delta = w[i + 1] - w[i];
            if delta != delta_n {
                continue 'outer;
            }
        }
        if delta_0 != delta_n {
            return false;
        }
    }
    true*/
}

pub fn land_and_river<R: Rng>(spec: &Spec, rng: &mut R) -> (Land, Vec<Coord>) {
    let perlin2 = Perlin2::new(rng);
    let zoom = 0.03;
    let height_weight = 1000.;
    let scale_coord = |coord: Coord| (coord.x as f64 * zoom, coord.y as f64 * zoom);
    let land_cells = Grid::new_fn(spec.size, |coord| LandCell {
        height: (perlin2.noise(scale_coord(coord)).abs() * height_weight) + 1.,
    });
    let land = Land {
        cells: land_cells,
        height_diff: 0.,
    };
    let river = land.plot_river();
    (land, river)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WorldCell1 {
    Land,
    Water,
}

fn world_grid1_from_river(size: Size, river: &[Coord]) -> Grid<WorldCell1> {
    let mut grid = Grid::new_copy(size, WorldCell1::Land);
    for &coord in river {
        *grid.get_checked_mut(coord) = WorldCell1::Water;
    }
    grid
}

fn world_grid1_widen_river(grid: &mut Grid<WorldCell1>) {
    let mut new_grid = grid.clone();
    for coord in grid.coord_iter() {
        for d in Direction::all() {
            let neighbour_coord = coord + d.coord();
            if let Some(&WorldCell1::Water) = grid.get(neighbour_coord) {
                *new_grid.get_checked_mut(coord) = WorldCell1::Water;
            }
        }
    }
    *grid = new_grid;
}

fn world_grid1_validate_no_loops(grid: &Grid<WorldCell1>) -> bool {
    let mut seen = HashSet::new();
    let mut region_counter = 0;
    for (coord, &cell) in grid.enumerate() {
        if let WorldCell1::Water = cell {
            continue;
        }
        if !seen.insert(coord) {
            continue;
        }
        let mut to_visit = VecDeque::new();
        to_visit.push_front(coord);
        while let Some(coord) = to_visit.pop_back() {
            for d in CardinalDirection::all() {
                let neighbour_coord = coord + d.coord();
                if let Some(&WorldCell1::Land) = grid.get(neighbour_coord) {
                    if seen.insert(neighbour_coord) {
                        to_visit.push_front(neighbour_coord);
                    }
                }
            }
        }
        region_counter += 1;
    }
    // the river should divide the land into 2 regions
    region_counter == 2
}

const TOWN_SIZE: Size = Size::new_u16(50, 50);

fn is_point_valid_for_river_town(grid: &Grid<WorldCell1>, coord: Coord) -> bool {
    if *grid.get_checked(coord) != WorldCell1::Water {
        return false;
    }
    let rect_grid = Grid::new_copy(TOWN_SIZE, ());
    let rect_top_left = coord - (rect_grid.size() / 2);
    let mut current = if let Some(cell) = grid.get(rect_top_left).cloned() {
        if let WorldCell1::Water = cell {
            return false;
        }
        WorldCell1::Land
    } else {
        return false;
    };
    let mut transition_count = 0;
    for relative_edge_coord in rect_grid.edge_coord_iter() {
        let coord = relative_edge_coord + rect_top_left;
        if let Some(&cell) = grid.get(coord) {
            if cell != current && current == WorldCell1::Land {
                transition_count += 1;
            }
            current = cell;
        } else {
            return false;
        }
    }
    // the rectangle intersects the river in two locations
    transition_count == 2
}

fn get_town_candidate_positions(grid: &Grid<WorldCell1>, river: &[Coord]) -> Vec<Vec<Coord>> {
    let town_position_range = 10;
    let town_indicies_approx = vec![river.len() / 4, (4 * river.len()) / 5];
    town_indicies_approx
        .into_iter()
        .map(|index_approx| {
            let mut candidates = Vec::new();
            for i in (index_approx - town_position_range)..(index_approx + town_position_range) {
                if let Some(&coord) = river.get(i) {
                    if is_point_valid_for_river_town(grid, coord) {
                        candidates.push(coord);
                    }
                }
            }
            candidates
        })
        .collect()
}

fn make_towns<R: Rng>(
    grid: &Grid<WorldCell1>,
    town_candidate_positions: &Vec<Vec<Coord>>,
    rng: &mut R,
) -> Option<(Grid<WorldCell1>, Vec<Coord>)> {
    let mut output = grid.clone();
    let mut town_positions = Vec::new();
    for (i, candidates) in town_candidate_positions.into_iter().enumerate() {
        let &_centre = if i == 0 {
            candidates.first().unwrap()
        } else {
            candidates.last().unwrap()
        };
        let &centre = candidates.choose(rng).unwrap();
        town_positions.push(centre);
        let rect_grid = Grid::new_copy(TOWN_SIZE, ());
        let rect_top_left = centre - (rect_grid.size() / 2);
        for relative_coord in rect_grid.coord_iter() {
            let coord = relative_coord + rect_top_left;
            let cell = output.get_checked_mut(coord);
            *cell = WorldCell1::Water;
        }
    }
    Some((output, town_positions))
}

fn convex_hull(points: &[(f64, f64)]) -> Vec<(f64, f64)> {
    use std::f64::consts;
    fn left_most_point(points: &[(f64, f64)]) -> (f64, f64) {
        let mut left_most = None;
        let mut x_min = f64::MAX;
        for &(x, y) in points {
            if x < x_min {
                x_min = x;
                left_most = Some((x, y));
            }
        }
        left_most.unwrap()
    }
    fn normalize_angle(mut radians: f64) -> f64 {
        while radians > consts::PI {
            radians -= consts::PI * 2.;
        }
        while radians <= -consts::PI {
            radians += consts::PI * 2.;
        }
        radians
    }
    fn get_next_point(
        points: &[(f64, f64)],
        current: (f64, f64),
        prev_angle: f64,
    ) -> ((f64, f64), f64) {
        let mut max_angle = f64::MIN;
        let mut best_point = None;
        for &point in points {
            if point == current {
                continue;
            }
            let current_to_point = (point.0 - current.0, point.1 - current.1);
            let angle = normalize_angle(
                current_to_point.1.atan2(current_to_point.0) - prev_angle
                    + std::f64::consts::FRAC_PI_2,
            );
            if angle > max_angle {
                max_angle = angle;
                best_point = Some(point);
            }
        }
        (
            best_point.unwrap(),
            normalize_angle(max_angle + prev_angle - std::f64::consts::FRAC_PI_2),
        )
    }
    let start = left_most_point(points);
    let mut current = start;
    let mut hull = vec![current];
    let mut prev_angle = std::f64::consts::FRAC_PI_2;
    loop {
        let (next_point, angle) = get_next_point(points, current, prev_angle);
        prev_angle = angle;
        if next_point == start {
            break;
        }
        hull.push(next_point);
        current = next_point;
    }
    hull
}

#[derive(Clone)]
pub struct Blob {
    pub border: Vec<Coord>,
    pub inside: Vec<Coord>,
}

pub fn blob<R: Rng>(coord: Coord, size: Size, rng: &mut R) -> Blob {
    let n = 400;
    let radius = 0.5;
    let mut points = Vec::new();
    for _ in 0..n {
        let angle = Radians(rng.gen::<f64>() * (2.0 * std::f64::consts::PI));
        let length = rng.gen::<f64>() * radius;
        let vector = Radial { length, angle };
        let Cartesian { x, y } = vector.to_cartesian();
        points.push((
            (x + radius) * size.width() as f64 * 2.0,
            (y + radius) * size.height() as f64 * 2.0,
        ));
    }
    let ch = convex_hull(&points);
    let mut ch_coords = ch
        .into_iter()
        .map(|(x, y)| Coord::new(x as i32, y as i32) + coord - (size.to_coord().unwrap()))
        .collect::<Vec<_>>();
    ch_coords.push(ch_coords[0]);
    let border_set = ch_coords
        .windows(2)
        .flat_map(|w| line_2d::coords_between_cardinal(w[0], w[1]))
        .collect::<HashSet<_>>();
    let border = border_set.iter().cloned().collect::<Vec<_>>();
    let centre = coord;
    let mut to_visit = VecDeque::new();
    to_visit.push_front(centre);
    let mut seen = HashSet::new();
    seen.insert(centre);
    while let Some(coord) = to_visit.pop_back() {
        for d in CardinalDirection::all() {
            let neighbour_coord = coord + d.coord();
            if !border_set.contains(&neighbour_coord) {
                if seen.insert(neighbour_coord) {
                    to_visit.push_front(neighbour_coord);
                }
            }
        }
    }
    let inside = seen.into_iter().collect::<Vec<_>>();
    Blob { border, inside }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WaterType {
    Ocean,
    River,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WorldCell2 {
    Water(WaterType),
    Land,
}

pub struct World2 {
    pub grid: Grid<WorldCell2>,
    pub spawn: Coord,
    pub ocean_x_ofset: u32,
    pub lake_centre: Coord,
    pub swamp_centre: Coord,
    pub city_centre: Coord,
    pub city_blob: Blob,
    pub gate: Vec<Coord>,
}

fn make_world_grid2<R: Rng>(
    spec: &Spec,
    river: &[Coord],
    town_positions: &[Coord],
    rng: &mut R,
) -> World2 {
    let left_padding = 100;
    let right_land_padding = 20;
    let right_ocean_padding = 50;
    let top_padding = 200;
    let bottom_padding = 200;
    let zoom = 3;
    let lake_radius = 35;
    let size = spec.size * zoom
        + Size::new(
            left_padding + right_land_padding + right_ocean_padding,
            top_padding + bottom_padding,
        );
    let scale_coord =
        |coord: Coord| (coord * zoom as i32) + Coord::new(left_padding as i32, top_padding as i32);
    let mut grid = Grid::new_copy(size, WorldCell2::Land);
    let lake_coord_unscaled = river[0] + Coord::new(-(lake_radius as i32) / zoom as i32, 0);
    let river_end_unscaled = (*river.last().unwrap())
        + Coord::new(
            (right_land_padding + right_ocean_padding) as i32 / zoom as i32,
            0,
        );
    let mut river = river.into_iter().cloned().collect::<VecDeque<_>>();
    river.push_front(lake_coord_unscaled);
    river.push_back(river_end_unscaled);
    let river = river.into_iter().collect::<Vec<_>>();
    for w in river.windows(2) {
        for coord in line_2d::coords_between(scale_coord(w[0]), scale_coord(w[1])) {
            *grid.get_checked_mut(coord) = WorldCell2::Water(WaterType::River);
        }
    }
    let widen_river = |grid: Grid<WorldCell2>| {
        let mut output = grid.clone();
        for (coord, &cell) in grid.enumerate() {
            if cell == WorldCell2::Water(WaterType::River) {
                for d in Direction::all() {
                    if let Some(output_cell) = output.get_mut(coord + d.coord()) {
                        *output_cell = WorldCell2::Water(WaterType::River);
                    }
                }
            }
        }
        output
    };
    for _ in 0..6 {
        grid = widen_river(grid);
    }
    let lake_coord = scale_coord(lake_coord_unscaled);
    let lake = blob(lake_coord, Size::new(lake_radius, lake_radius), rng);
    for &coord in &lake.inside {
        *grid.get_checked_mut(coord) = WorldCell2::Water(WaterType::River);
    }
    let spawn = lake_coord;
    let town_size = TOWN_SIZE * zoom;
    let mut pool_centres = Vec::new();
    let mut town_blobs = Vec::new();
    for &town_coord_unscaled in town_positions {
        let town_coord = scale_coord(town_coord_unscaled);
        pool_centres.push(town_coord);
        let radius = town_size / 2;
        let town_blob = blob(town_coord, radius, rng);
        for &coord in &town_blob.inside {
            *grid.get_checked_mut(coord) = WorldCell2::Water(WaterType::River);
        }
        town_blobs.push(town_blob);
    }
    let city_blob = town_blobs[1].clone();
    let swamp_centre = pool_centres[0];
    let city_centre = pool_centres[1];
    let ocean_centre = Coord::new(grid.width() as i32, grid.height() as i32 / 2);
    let ocean_height = grid.height();
    let ocean_blob = blob(
        ocean_centre,
        Size::new(right_ocean_padding, ocean_height),
        rng,
    );
    for &coord in &ocean_blob.inside {
        if let Some(cell) = grid.get_mut(coord) {
            *cell = WorldCell2::Water(WaterType::Ocean);
        }
    }
    let city_blob_set = city_blob.inside.iter().cloned().collect::<HashSet<_>>();
    let gate_centre = {
        let mut gate_centre = None;
        for &coord in &river {
            if city_blob_set.contains(&scale_coord(coord)) {
                gate_centre = Some(scale_coord(coord));
            }
        }
        gate_centre.unwrap()
    };
    let gate = city_blob
        .border
        .iter()
        .cloned()
        .filter(|coord| coord.distance2(gate_centre) < 400)
        .collect::<Vec<_>>();
    //let spawn = scale_coord(river_end_unscaled) - Coord::new(right_ocean_padding as i32 + 10, 0);
    World2 {
        spawn,
        ocean_x_ofset: grid.width() - right_ocean_padding,
        grid,
        lake_centre: lake_coord,
        swamp_centre,
        city_centre,
        city_blob,
        gate,
    }
}

pub struct WaterDistanceMap {
    pub distances: Grid<u32>,
}

impl WaterDistanceMap {
    fn new(world2: &Grid<WorldCell2>) -> Self {
        let max_distance = 30;
        let mut distances = Grid::new_copy(world2.size(), 0);
        let mut seen = HashSet::new();
        let mut to_visit = VecDeque::new();
        for (coord, &cell) in world2.enumerate() {
            if let WorldCell2::Water(_) = cell {
                if Direction::all().all(|d| {
                    if let Some(WorldCell2::Water(_)) = world2.get(coord + d.coord()) {
                        true
                    } else {
                        false
                    }
                }) {
                    continue;
                }
                seen.insert(coord);
                to_visit.push_front(coord);
            } else {
                *distances.get_checked_mut(coord) = std::u32::MAX;
            }
        }
        while let Some(coord) = to_visit.pop_back() {
            for d in Direction::all() {
                let neighbour_coord = coord + d.coord();
                if let Some(WorldCell2::Land) = world2.get(neighbour_coord) {
                    if seen.insert(neighbour_coord) {
                        let distance = *distances.get_checked(coord) + 1;
                        if distance < max_distance {
                            *distances.get_checked_mut(neighbour_coord) = distance;
                            to_visit.push_front(neighbour_coord);
                        }
                    }
                }
            }
        }
        Self { distances }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WorldCell3 {
    Ground,
    TownGround,
    Floor,
    Water(WaterType),
    Wall,
    Gate,
    Door,
    StairsDown,
    StairsUp,
    Grave,
}

pub struct World3 {
    pub grid: Grid<WorldCell3>,
    pub spawn: Coord,
    pub boat_spawn: Coord,
    pub boat_heading: Radians,
    pub your_door: Coord,
    pub unimportant_npc_spawns: HashSet<Coord>,
    pub grave_pool: Vec<Coord>,
    pub npc_spawns: Vec<Coord>,
    pub junk_spawns: Vec<Coord>,
    pub island_coords: Vec<Coord>,
    pub inside_coords: Vec<Coord>,
    pub shop_coords: Vec<Coord>,
    pub building_coords: Vec<Coord>,
}

impl World3 {
    fn from_world2<R: Rng>(world2: &World2, num_graves: u32, rng: &mut R) -> Option<World3> {
        let mut npc_spawns = Vec::new();
        let mut grid = world2.grid.map_ref(|cell| match cell {
            WorldCell2::Land => WorldCell3::Ground,
            WorldCell2::Water(w) => WorldCell3::Water(*w),
        });
        let mut junk_spawns = Vec::new();
        let mut inside_coords = Vec::new();
        let mut shop_coords = Vec::new();
        let mut island_coords_set = HashSet::new();
        let mut building_coords_set = HashSet::new();
        let lake_bottom = {
            let mut c = world2.lake_centre;
            let c = loop {
                if let WorldCell2::Water(_) = *world2.grid.get_checked(c) {
                    c += Coord::new(0, 1);
                    continue;
                }
                break c;
            };
            // padding in case the lack goes down further on one side
            c + Coord::new(0, 7)
        };
        let pier_length = 15;
        {
            // pier
            let mut c = lake_bottom;
            for _ in 0..pier_length {
                *grid.get_checked_mut(c) = WorldCell3::Floor;
                *grid.get_checked_mut(c - Coord::new(1, 0)) = WorldCell3::Floor;
                c -= Coord::new(0, 1);
            }
            npc_spawns.push(lake_bottom - Coord::new(0, pier_length - 3));
            npc_spawns.push(lake_bottom - Coord::new(1, pier_length - 3));
        }
        let boat_spawn = lake_bottom - Coord::new(0, pier_length + 2);
        let boat_heading = Radians(std::f64::consts::FRAC_PI_2);
        let mut unimportant_npc_spawns = HashSet::new();
        let grave_pool = {
            // graveyard
            let distance_from_town = 30;
            if num_graves > 0 {
                for i in 0..distance_from_town {
                    let c = lake_bottom + Coord::new(0, i);
                    *grid.get_checked_mut(c) = WorldCell3::Floor;
                }
            }
            let centre = lake_bottom + Coord::new(0, distance_from_town);
            let clear_blob = blob(centre, Size::new(8, 8), rng);
            for c in clear_blob.inside {
                *grid.get_checked_mut(c) = WorldCell3::Floor;
            }
            let pool_blob = blob(centre, Size::new(4, 4), rng);
            let mut grave_pool = Vec::new();
            for c in pool_blob.inside {
                *grid.get_checked_mut(c) = WorldCell3::Water(WaterType::Ocean);
                grave_pool.push(c);
            }
            let grave_blob = blob(centre, Size::new(6, 6), rng);
            let mut grave_candidates = Vec::new();
            for c in grave_blob.inside {
                if *grid.get_checked(c) == WorldCell3::Floor && c.x % 2 == 0 {
                    grave_candidates.push(c);
                }
            }
            grave_candidates.shuffle(rng);
            for c in grave_candidates.into_iter().take(num_graves as usize) {
                *grid.get_checked_mut(c) = WorldCell3::Grave;
            }
            grave_pool
        };
        let (spawn, your_door) = {
            // lake town
            let lake_town_area = blob(lake_bottom, Size::new(30, 10), rng);
            for coord in lake_town_area.inside {
                if let WorldCell3::Ground = *grid.get_checked(coord) {
                    *grid.get_checked_mut(coord) = WorldCell3::TownGround;
                }
            }
            let road_left = -20;
            let road_right = 20;
            let mut road_min = road_right;
            let mut road_max = road_left;
            let mut start_coord = None;
            let mut your_door = None;
            let mut max_dist = 0;
            // houses
            let mut num_houses = 0;
            while num_houses < 2 {
                let num_house_attemps = 50;
                'outer: for _ in 0..num_house_attemps {
                    let width = rng.gen_range(6..=8);
                    let height = rng.gen_range(6..=7);
                    let size = Size::new(width, height);
                    let x = rng.gen_range(road_left..(road_right - width as i32));
                    let y = if rng.gen::<bool>() {
                        // below road
                        rng.gen_range(1i32..4i32)
                    } else {
                        // above road
                        rng.gen_range((-(height as i32) - 3)..(-(height as i32)))
                    };
                    let coord = Coord::new(x, y) + lake_bottom;
                    let house_grid = Grid::new_copy(size, ());
                    for offset in house_grid.coord_iter() {
                        let c = coord + offset;
                        if let WorldCell3::TownGround = *grid.get_checked(c) {
                        } else {
                            continue 'outer;
                        }
                    }
                    let house_grid = Grid::new_copy(size - Size::new(2, 2), ());
                    for offset in house_grid.coord_iter() {
                        let c = coord + offset + Coord::new(1, 1);
                        *grid.get_checked_mut(c) = WorldCell3::Floor;
                    }
                    for offset in house_grid.edge_coord_iter() {
                        let c = coord + offset + Coord::new(1, 1);
                        *grid.get_checked_mut(c) = WorldCell3::Wall;
                    }
                    let door_offset = rng.gen_range(2..(width as i32 - 2)) + x;
                    let door_y = if y < 0 { y + height as i32 - 2 } else { y + 1 };
                    let door_coord = Coord::new(door_offset, door_y) + lake_bottom;
                    road_min = road_min.min(door_offset);
                    road_max = road_max.max(door_offset);
                    *grid.get_checked_mut(door_coord) = WorldCell3::Door;
                    if y < 0 {
                        for i in (door_coord.y + 1)..lake_bottom.y {
                            let c = Coord::new(door_coord.x, i);
                            *grid.get_checked_mut(c) = WorldCell3::Floor;
                        }
                        if door_offset.abs() > max_dist {
                            max_dist = door_offset.abs();
                            start_coord = Some(door_coord - Coord::new(0, 2));
                            your_door = Some(door_coord);
                        }
                        if rng.gen::<f64>() < 0.3 {
                            unimportant_npc_spawns.insert(door_coord + Coord::new(0, 2));
                        }
                    } else {
                        for i in (lake_bottom.y + 1)..door_coord.y {
                            let c = Coord::new(door_coord.x, i);
                            *grid.get_checked_mut(c) = WorldCell3::Floor;
                        }
                        if door_offset.abs() > max_dist {
                            max_dist = door_offset.abs();
                            start_coord = Some(door_coord + Coord::new(0, 2));
                            your_door = Some(door_coord);
                        }
                        if rng.gen::<f64>() < 0.3 {
                            unimportant_npc_spawns.insert(door_coord - Coord::new(0, 2));
                        }
                    };
                    num_houses += 1;
                }
            }
            // road
            for i in road_min..=road_max {
                let c = lake_bottom + Coord::new(i, 0);
                *grid.get_checked_mut(c) = WorldCell3::Floor;
            }
            (start_coord.unwrap(), your_door.unwrap())
        };
        {
            {
                // inn
                let angle = Radians(rng.gen::<f64>() * (2.0 * std::f64::consts::PI));
                let distance = rng.gen::<f64>() * TOWN_SIZE.width() as f64 / 2.;
                let coord = Radial {
                    angle,
                    length: distance,
                }
                .to_cartesian()
                .to_coord_round_nearest()
                    + world2.swamp_centre;
                let platform_size = Size::new(10, 10);
                let platform_coord = coord - platform_size.to_coord().unwrap() / 2;
                for c in platform_size.coord_iter_row_major() {
                    let coord = c + platform_coord;
                    *grid.get_checked_mut(coord) = WorldCell3::Floor;
                    if c.x > 2 {
                        inside_coords.push(coord);
                    }
                }
                shop_coords.push(Coord::new(7, 7) + platform_coord);
                let building_size = platform_size - Size::new(2, 0);
                let building_coord = platform_coord + Coord::new(2, 0);
                let building_grid = Grid::new_copy(building_size, ());
                for c in building_grid.edge_coord_iter() {
                    *grid.get_checked_mut(c + building_coord) = WorldCell3::Wall;
                }

                // inn pier
                for i in 1..10 {
                    let c = Coord::new(-i, 3) + platform_coord;
                    *grid.get_checked_mut(c) = WorldCell3::Floor;
                    let c = Coord::new(-i, 4) + platform_coord;
                    *grid.get_checked_mut(c) = WorldCell3::Floor;
                }
                junk_spawns.push(building_coord + Coord::new(-1, 5));
                *grid.get_checked_mut(building_coord + Coord::new(0, 7)) = WorldCell3::Door;
                let npc_coord = platform_coord + Coord::new(-8, 4);
                npc_spawns.push(npc_coord);
            }
            // swamp
            let mut num_islands = 0;
            let num_island_attempts = 200;
            let mut npc_candidates = Vec::new();
            'outer: for _ in 0..num_island_attempts {
                let angle = Radians(rng.gen::<f64>() * (2.0 * std::f64::consts::PI));
                let distance = rng.gen::<f64>() * TOWN_SIZE.width() as f64 * 3. / 2.;
                let coord = Radial {
                    angle,
                    length: distance,
                }
                .to_cartesian()
                .to_coord_round_nearest()
                    + world2.swamp_centre;
                let boundary = blob(coord, Size::new(18, 18), rng);
                for c in boundary.inside {
                    if let Some(WorldCell3::Water(_)) = grid.get(c) {
                    } else {
                        continue 'outer;
                    }
                }
                let num_island_parts = rng.gen_range(2..=4);
                let mut island_coords = Vec::new();
                for _ in 0..num_island_parts {
                    let centre = coord + Coord::new(rng.gen_range(-2..=2), rng.gen_range(-2..=2));
                    let size = Size::new(rng.gen_range(3..8), rng.gen_range(3..8));
                    let part = blob(centre, size, rng);
                    for c in part.inside {
                        *grid.get_checked_mut(c) = WorldCell3::Ground;
                        island_coords.push(c);
                        island_coords_set.insert(c);
                    }
                }
                for _ in 0..rng.gen_range(1..=1) {
                    let beach_coords = island_coords
                        .iter()
                        .cloned()
                        .filter(|coord| {
                            for d in Direction::all() {
                                let cell = *grid.get_checked(coord + d.coord());
                                if cell == WorldCell3::Water(WaterType::River)
                                    || cell == WorldCell3::Floor
                                {
                                    return true;
                                }
                            }
                            false
                        })
                        .collect::<Vec<_>>();
                    for c in beach_coords {
                        let cell = grid.get_checked_mut(c);
                        *cell = WorldCell3::Floor;
                        npc_candidates.push(c);
                        island_coords_set.remove(&c);
                    }
                }
                num_islands += 1;
            }
            if num_islands < 0 {
                return None;
            }
            let npc_coord = *npc_candidates.choose(rng).unwrap();
            npc_spawns.push(npc_coord);
        }
        let _spawn = {
            // city
            {
                // gate
                for &c in &world2.gate {
                    let cell = grid.get_checked_mut(c);
                    if *cell == WorldCell3::Water(WaterType::River) {
                        *cell = WorldCell3::Gate;
                    }
                }
            }
            // building grid
            let grid_size = Size::new(4, 4);
            let building_size_including_padding = Size::new(25, 25);
            let city_size = Size::new(
                grid_size.width() * building_size_including_padding.width(),
                grid_size.height() * building_size_including_padding.height(),
            );
            let city_coord = world2.city_centre - city_size.to_coord().unwrap() / 2;
            let mut block_coords = grid_size
                .coord_iter_row_major()
                .filter(|&c| {
                    !(c == Coord::new(0, 0)
                        || c == Coord::new(grid_size.width() as i32 - 1, 0)
                        || c == Coord::new(0, grid_size.height() as i32 - 1)
                        || c == Coord::new(
                            grid_size.width() as i32 - 1,
                            grid_size.height() as i32 - 1,
                        ))
                })
                .collect::<Vec<_>>();
            block_coords.shuffle(rng);
            for _ in 0..2 {
                block_coords.pop();
            }

            let inn_block_coord = block_coords.pop().unwrap();
            let mut stairs_candidates = Vec::new();
            for c in block_coords {
                let padding = rng.gen_range(3..6);
                let coord = Coord {
                    x: c.x * building_size_including_padding.width() as i32,
                    y: c.y * building_size_including_padding.height() as i32,
                } + Coord::new(padding, padding)
                    + city_coord;
                let size =
                    building_size_including_padding - Size::new(padding as u32, padding as u32) * 2;
                let g = Grid::new_copy(size, ());
                for c in g.coord_iter() {
                    let cell = grid.get_checked_mut(c + coord);
                    *cell = WorldCell3::Floor;
                }
                for c in g.edge_coord_iter() {
                    let cell = grid.get_checked_mut(c + coord);
                    *cell = WorldCell3::Wall;
                }
                for i in 0..(size.height() as i32) {
                    let c = Coord::new(size.width() as i32 / 2, i);
                    let cell = grid.get_checked_mut(c + coord);
                    *cell = WorldCell3::Wall;
                }
                for i in 0..(size.width() as i32) {
                    let c = Coord::new(i, size.height() as i32 / 2);
                    let cell = grid.get_checked_mut(c + coord);
                    *cell = WorldCell3::Wall;
                }
                *grid.get_checked_mut(
                    coord + Coord::new(size.width() as i32 / 4, size.height() as i32 / 2),
                ) = WorldCell3::Door;
                *grid.get_checked_mut(
                    coord + Coord::new(3 * size.width() as i32 / 4, size.height() as i32 / 2),
                ) = WorldCell3::Door;
                *grid.get_checked_mut(
                    coord + Coord::new(size.width() as i32 / 2, size.height() as i32 / 4),
                ) = WorldCell3::Door;
                *grid.get_checked_mut(
                    coord + Coord::new(size.width() as i32 / 2, 3 * size.height() as i32 / 4),
                ) = WorldCell3::Door;
                let mut outer_door_candidates = vec![
                    Coord::new(size.width() as i32 / 4, 0),
                    Coord::new(size.width() as i32 / 4, size.height() as i32 - 1),
                    Coord::new(3 * size.width() as i32 / 4, 0),
                    Coord::new(3 * size.width() as i32 / 4, size.height() as i32 - 1),
                    Coord::new(0, size.height() as i32 / 4),
                    Coord::new(0, 3 * size.height() as i32 / 4),
                    Coord::new(size.width() as i32 - 1, size.height() as i32 / 4),
                    Coord::new(size.width() as i32 - 1, 3 * size.height() as i32 / 4),
                ];
                outer_door_candidates.shuffle(rng);
                for _ in 0..rng.gen_range(1..=3) {
                    let door_coord = outer_door_candidates.pop().unwrap() + coord;
                    *grid.get_checked_mut(door_coord) = WorldCell3::Door;
                }
                let mut bomb_candidates = g.edge_coord_iter().collect::<Vec<_>>();
                bomb_candidates.shuffle(rng);
                for bomb_coord in bomb_candidates.into_iter().take(rng.gen_range(1..=3)) {
                    let size = Size::new(rng.gen_range(4..8), rng.gen_range(4..8));
                    let bomb = blob(bomb_coord, size, rng);
                    for c in bomb.inside {
                        let cell = grid.get_checked_mut(c + coord);
                        if *cell != WorldCell3::Water(WaterType::River) {
                            *cell = WorldCell3::TownGround;
                        }
                    }
                }
                let stairs_candidates_here = vec![
                    coord + Coord::new(size.width() as i32 / 4, size.height() as i32 / 4),
                    coord + Coord::new(3 * size.width() as i32 / 4, size.height() as i32 / 4),
                    coord + Coord::new(size.width() as i32 / 4, 3 * size.height() as i32 / 4),
                    coord + Coord::new(3 * size.width() as i32 / 4, 3 * size.height() as i32 / 4),
                ];
                let stair_coord = *stairs_candidates_here.choose(rng).unwrap();
                stairs_candidates.push(stair_coord);
                for c in g.coord_iter() {
                    let cell = *grid.get_checked(c + coord);
                    if cell == WorldCell3::Floor {
                        building_coords_set.insert(c + coord);
                    }
                }
                building_coords_set.remove(&stair_coord);
            }
            stairs_candidates.shuffle(rng);
            for c in stairs_candidates.into_iter().take(4) {
                let cell = grid.get_checked_mut(c);
                *cell = WorldCell3::StairsDown;
            }
            {
                // inn
                let inn_coord = Coord::new(
                    inn_block_coord.x * building_size_including_padding.width() as i32,
                    inn_block_coord.y * building_size_including_padding.height() as i32,
                ) + city_coord
                    + Coord::new(15, 14);
                let platform_size = Size::new(10, 10);
                let platform_coord = inn_coord - platform_size.to_coord().unwrap() / 2;
                for c in platform_size.coord_iter_row_major() {
                    let coord = c + platform_coord;
                    *grid.get_checked_mut(c + platform_coord) = WorldCell3::Floor;
                    if c.x > 2 {
                        inside_coords.push(coord);
                    }
                }
                shop_coords.push(Coord::new(7, 7) + platform_coord);
                let building_size = platform_size - Size::new(2, 0);
                let building_coord = platform_coord + Coord::new(2, 0);
                let building_grid = Grid::new_copy(building_size, ());
                for c in building_grid.edge_coord_iter() {
                    *grid.get_checked_mut(c + building_coord) = WorldCell3::Wall;
                }
                for i in 1..10 {
                    let c = Coord::new(-i, 3) + platform_coord;
                    *grid.get_checked_mut(c) = WorldCell3::Floor;
                    let c = Coord::new(-i, 4) + platform_coord;
                    *grid.get_checked_mut(c) = WorldCell3::Floor;
                }
                *grid.get_checked_mut(building_coord + Coord::new(0, 7)) = WorldCell3::Door;
                let inn_centre = building_coord + Coord::new(2, 4);
                let npc_coord = platform_coord + Coord::new(-8, 4);
                npc_spawns.push(npc_coord);
                //*grid.get_checked_mut(inn_centre + Coord::new(0, 2)) = WorldCell3::StairsDown;
                inn_centre
            }
        };
        //let spawn = world2.swamp_centre;
        //let spawn = Coord { x: 567, y: 274 };
        //let boat_spawn = spawn + Coord::new(-10, -4);
        Some(Self {
            grid,
            spawn,
            boat_spawn,
            boat_heading,
            your_door,
            unimportant_npc_spawns,
            grave_pool,
            npc_spawns,
            junk_spawns,
            inside_coords,
            shop_coords,
            island_coords: island_coords_set.into_iter().collect(),
            building_coords: building_coords_set.into_iter().collect(),
        })
    }
}

pub struct Terrain {
    pub land: Land,
    pub river: Vec<Coord>,
    pub world1: Grid<WorldCell1>,
    pub world2: World2,
    pub world3: World3,
    pub water_distance_map: WaterDistanceMap,
    pub viz_coord: Coord,
    pub viz_size: Size,
}

pub fn generate<R: Rng>(spec: &Spec, rng: &mut R) -> Terrain {
    loop {
        let (land, river) = loop {
            let (land, river) = land_and_river(spec, rng);
            if validate_river(&river) {
                break (land, river);
            }
        };
        let mut world1 = world_grid1_from_river(spec.size, &river);
        world_grid1_widen_river(&mut world1);
        world_grid1_widen_river(&mut world1);

        let town_candidate_positions = get_town_candidate_positions(&world1, &river);
        if town_candidate_positions.iter().any(|v| v.is_empty()) {
            continue;
        }
        let (world1, town_positions) = if let Some((world1, town_positions)) =
            make_towns(&world1, &town_candidate_positions, rng)
        {
            (world1, town_positions)
        } else {
            continue;
        };
        if !world_grid1_validate_no_loops(&world1) {
            continue;
        }
        let mut world2 = make_world_grid2(spec, &river, &town_positions, rng);
        let viz_size = Size::new(200, 160);
        let world3 = if let Some(world3) = World3::from_world2(&world2, spec.num_graves, rng) {
            world3
        } else {
            continue;
        };
        for &c in world3.grave_pool.iter() {
            *world2.grid.get_checked_mut(c) = WorldCell2::Water(WaterType::River);
        }
        let water_distance_map = WaterDistanceMap::new(&world2.grid);
        let viz_coord = world3.spawn - (viz_size.to_coord().unwrap() / 2);
        break Terrain {
            land,
            river,
            world1,
            world2,
            world3,
            water_distance_map,
            viz_coord,
            viz_size,
        };
    }
}

pub struct Dungeon {
    pub spawn: Coord,
    pub grid: Grid<DungeonCell>,
    pub destination: Coord,
    pub other_room_centres: Vec<Coord>,
}

#[derive(Debug, Clone, Copy)]
pub enum DungeonCell {
    Door,
    Wall,
    Floor,
}

pub fn generate_dungeon<R: Rng>(size: Size, rng: &mut R) -> Dungeon {
    use rooms_and_corridors::*;
    let RoomsAndCorridorsLevel {
        map,
        player_spawn,
        destination,
        other_room_centres,
    } = RoomsAndCorridorsLevel::generate(size, rng);
    let grid = map.map_ref(|cell| match cell {
        RoomsAndCorridorsCell::Door => DungeonCell::Door,
        RoomsAndCorridorsCell::Wall => DungeonCell::Wall,
        RoomsAndCorridorsCell::Floor => DungeonCell::Floor,
    });
    Dungeon {
        grid,
        spawn: player_spawn,
        destination,
        other_room_centres,
    }
}
