use gridbugs::{
    coord_2d::{Coord, Size},
    direction::CardinalDirection,
    grid_2d::Grid,
    perlin2::Perlin2,
};
use rand::Rng;
use std::{
    cmp::Ordering,
    collections::{BinaryHeap, HashMap},
};

pub struct Spec {
    pub size: Size,
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
        self.cells
            .get(coord)
            .map(|land_cell| self.base_height_on_row(coord.x as usize) + land_cell.height)
    }
    pub fn plot_river(&self) -> Vec<Coord> {
        let mut best_cost = f64::MAX;
        let mut best = Vec::new();
        for i in 0..self.cells.height() as usize {
            let (path, cost) = self.plot_river_from_row(i);
            //eprintln!("{}: {}", i, cost);
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
            //eprintln!("{:?}", cell);
            if cell.coord.x as u32 == self.cells.width() - 1 {
                end = Some(cell);
                break;
            }
            for d in CardinalDirection::all() {
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
    let n = 4;
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
    true
}

pub fn land_and_river<R: Rng>(spec: &Spec, rng: &mut R) -> (Land, Vec<Coord>) {
    let perlin2 = Perlin2::new(rng);
    let zoom = 0.05;
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

pub struct Terrain {
    pub land: Land,
    pub river: Vec<Coord>,
}

pub fn generate<R: Rng>(spec: &Spec, rng: &mut R) -> Terrain {
    let (land, river) = loop {
        let (land, river) = land_and_river(spec, rng);
        if validate_river(&river) {
            break (land, river);
        }
        eprintln!("x");
    };
    Terrain { land, river }
}
