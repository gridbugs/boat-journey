pub use crate::world::spatial::{Layer, Location};
use gridbugs::{coord_2d::Coord, entity_table::declare_entity_module, line_2d::InfiniteStepIter};
use serde::{Deserialize, Serialize};
use vector::{Radial, Radians};

declare_entity_module! {
    components {
        tile: Tile,
        boat: Boat,
        solid: (),
        part_of_boat: (),
        door_state: DoorState,
        opacity: u8,
        boat_controls: (),
        ocean: (),
    }
}
pub use components::{Components, EntityData, EntityUpdate};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Tile {
    Player,
    BoatEdge,
    BoatFloor,
    Water1,
    Water2,
    Floor,
    Wall,
    DoorClosed,
    DoorOpen,
    Rock,
    Board,
    BoatControls,
    Tree,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Boat {
    heading: Radians,
    movement_iter: InfiniteStepIter,
}

impl Boat {
    fn sync_movement_iter(&self) -> Self {
        let mut ret = self.clone();
        let movement_delta = Radial {
            length: 1000f64,
            angle: Radians(self.heading.0 - std::f64::consts::FRAC_PI_2),
        }
        .to_cartesian()
        .to_coord_round_nearest();
        ret.movement_iter = InfiniteStepIter::new(movement_delta);
        ret
    }

    pub fn new(heading: Radians) -> Self {
        Self {
            heading,
            movement_iter: InfiniteStepIter::new(Coord::new(1, 0)),
        }
        .sync_movement_iter()
    }

    #[must_use]
    pub fn add_heading(&self, delta: Radians) -> Self {
        let mut ret = self.clone();
        ret.heading.0 += delta.0;
        ret.sync_movement_iter()
    }

    pub fn heading(&self) -> Radians {
        self.heading
    }

    #[must_use]
    pub fn step(&self) -> (Self, Coord) {
        let mut ret = self.clone();
        let coord = ret.movement_iter.step().coord();
        (ret, coord)
    }

    #[must_use]
    pub fn step_backwards(&self) -> (Self, Coord) {
        let mut ret = self.clone();
        let coord = ret.movement_iter.step_back().coord();
        (ret, coord)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DoorState {
    Open,
    Closed,
}
