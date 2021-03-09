use crate::world::{Components, SpatialTable};
use crate::Entity;
use direction::{CardinalDirection, CardinalDirections};
use grid_2d::{Coord, Grid, Size};
use serde::{Deserialize, Serialize};
use std::collections::{HashSet, VecDeque};

#[derive(Debug, Serialize, Deserialize)]
pub struct Air {
    pressure: Grid<u8>,
    flow: Grid<Option<u32>>,
    queue: VecDeque<Coord>,
    seen: HashSet<Coord>,
}

const MAX_AIR: u8 = 6;

impl Air {
    pub fn new(size: Size) -> Self {
        let pressure = Grid::new_copy(size, MAX_AIR);
        let flow = Grid::new_copy(size, None);
        let queue = Default::default();
        let seen = Default::default();
        Self {
            pressure,
            flow,
            queue,
            seen,
        }
    }
    pub fn init(&mut self, spatial_table: &SpatialTable, components: &Components) {
        self.queue.clear();
        self.seen.clear();
        for (coord, layers) in spatial_table.enumerate() {
            if layers.floor.is_none() {
                self.queue.push_back(coord);
                self.seen.insert(coord);
            }
        }
        while let Some(coord) = self.queue.pop_front() {
            *self.pressure.get_checked_mut(coord) = 0;
            for direction in CardinalDirections {
                let nei_coord = coord + direction.coord();
                if let Some(layers) = spatial_table.layers_at(nei_coord) {
                    if self.seen.insert(nei_coord) {
                        if let Some(feature) = layers.feature {
                            if !components.solid.contains(feature) {
                                self.queue.push_back(nei_coord);
                            }
                        }
                    }
                }
            }
        }
    }

    pub fn update(
        &mut self,
        spatial_table: &SpatialTable,
        components: &Components,
    ) -> Vec<(Entity, CardinalDirection)> {
        let mut to_move = Vec::new();
        self.queue.clear();
        for (coord, flow_cell) in self.flow.enumerate_mut() {
            if spatial_table.layers_at_checked(coord).floor.is_none() {
                *flow_cell = Some(0);
                self.queue.push_back(coord);
            } else {
                *flow_cell = None;
            }
        }
        while let Some(coord) = self.queue.pop_front() {
            let current_flow = self.flow.get_checked(coord).unwrap();
            for direction in CardinalDirections {
                let nei_coord = coord + direction.coord();
                if let Some(flow_cell) = self.flow.get_mut(nei_coord) {
                    if flow_cell.is_some() {
                        continue;
                    }
                    if let Some(layers) = spatial_table.layers_at(nei_coord) {
                        if let Some(feature) = layers.feature {
                            if components.solid.contains(feature) {
                                continue;
                            }
                        }
                        *flow_cell = Some(current_flow + 1);
                        let pressure = self.pressure.get_checked_mut(nei_coord);
                        if *pressure > 0 {
                            *pressure = pressure.saturating_sub(1);
                            if let Some(character) = layers.character {
                                to_move.push((character, direction.opposite()));
                            }
                            if let Some(item) = layers.item {
                                to_move.push((item, direction.opposite()));
                            }
                        }
                        self.queue.push_back(nei_coord);
                    }
                }
            }
        }
        for (coord, air_cell) in self.pressure.enumerate_mut() {
            if self.flow.get_checked(coord).is_none() {
                *air_cell = MAX_AIR;
            }
        }
        to_move
    }

    pub fn has_air(&self, coord: Coord) -> bool {
        if let Some(cell) = self.pressure.get(coord) {
            *cell > 0
        } else {
            false
        }
    }

    pub fn has_flow(&self, coord: Coord) -> bool {
        if let Some(Some(flow)) = self.flow.get(coord) {
            *flow > 0
        } else {
            false
        }
    }
}
