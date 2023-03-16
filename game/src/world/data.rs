pub use crate::world::spatial::{Layer, Location};
use coord_2d::Coord;
use entity_table::declare_entity_module;
use line_2d::InfiniteStepIter;
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
        stairs_down: usize,
        stairs_up: (),
        ghost: (),
        unimportant_npc: (),
        threshold: (),
        grave: crate::Victory,
        npc: Npc,
        junk: Junk,
        inside: (),
        shop: usize,
        button: bool,
        gate: (),
        beast: (),
        destructible: (),
    }
}
pub use components::{Components, EntityData, EntityUpdate};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum Junk {
    RotaryPhone,
    BrokenTypewriter,
    PolaroidCamera,
    VhsTape,
    VinylRecord,
    CassettePlayer,
}

impl Junk {
    pub fn name(self) -> String {
        let s = match self {
            Self::RotaryPhone => "rotary telephone",
            Self::BrokenTypewriter => "broken typewriter",
            Self::PolaroidCamera => "polaroid camera",
            Self::VhsTape => "VHS tape",
            Self::VinylRecord => "vinyl record",
            Self::CassettePlayer => "cassette player",
        };
        s.to_string()
    }
    pub fn all() -> Vec<Junk> {
        vec![
            Junk::RotaryPhone,
            Junk::BrokenTypewriter,
            Junk::PolaroidCamera,
            Junk::VhsTape,
            Junk::VinylRecord,
            Junk::CassettePlayer,
        ]
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Hash)]
pub enum Tile {
    Player,
    BoatEdge,
    BoatFloor,
    Water1,
    Water2,
    Floor,
    BurntFloor,
    Wall,
    DoorClosed,
    DoorOpen,
    Rock,
    Board,
    BoatControls,
    Tree,
    StairsDown,
    StairsUp,
    Ghost,
    UnimportantNpc,
    Grave,
    Npc(Npc),
    Junk,
    Shop,
    Button,
    ButtonPressed,
    Beast,
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Meter {
    current: u32,
    max: u32,
}

impl Meter {
    pub fn new(current: u32, max: u32) -> Self {
        Self { current, max }
    }
    pub fn current_and_max(&self) -> (u32, u32) {
        (self.current, self.max)
    }
    pub fn current(&self) -> u32 {
        self.current
    }
    pub fn max(&self) -> u32 {
        self.max
    }
    pub fn set_current(&mut self, to: u32) {
        self.current = to.min(self.max);
    }
    pub fn decrease(&mut self, by: u32) {
        self.current = self.current.saturating_sub(by);
    }
    pub fn increase(&mut self, by: u32) {
        self.set_current(self.current + by);
    }
    pub fn set_max(&mut self, to: u32) {
        self.max = to;
        self.set_current(self.current);
    }
    pub fn is_empty(&self) -> bool {
        self.current == 0
    }
    pub fn is_full(&self) -> bool {
        self.current == self.max
    }
    pub fn fill(&mut self) {
        self.current = self.max;
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Hash)]
pub enum Npc {
    Soldier,
    Physicist,
    Beast,
    Ghost,
    Surgeon,
    Thief,
    Surveyor,
}

impl Npc {
    pub fn all() -> Vec<Self> {
        vec![
            Self::Soldier,
            Self::Physicist,
            Self::Beast,
            Self::Ghost,
            Self::Surgeon,
            Self::Thief,
            Self::Surveyor,
        ]
    }
    pub fn name(self) -> String {
        match self {
            Self::Soldier => format!("Soldier"),
            Self::Physicist => format!("Physicist"),
            Self::Beast => format!("Beast"),
            Self::Ghost => format!("Ghost"),
            Self::Surgeon => format!("Surgeon"),
            Self::Thief => format!("Thief"),
            Self::Surveyor => format!("Surveyor"),
        }
    }
    pub fn ability_name(self) -> String {
        match self {
            Self::Soldier => format!("Destroy"),
            Self::Physicist => format!("Blink"),
            Self::Beast => format!("Fear"),
            Self::Ghost => format!("Phase"),
            Self::Surgeon => format!("Heal"),
            Self::Thief => format!("Sneak"),
            Self::Surveyor => format!("Telescope"),
        }
    }
    pub fn ability_uses(self) -> u32 {
        match self {
            Self::Soldier => 2,
            Self::Physicist => 2,
            Self::Beast => 2,
            Self::Ghost => 2,
            Self::Surgeon => 2,
            Self::Thief => 2,
            Self::Surveyor => 2,
        }
    }
    pub fn text(self) -> String {
        let name = self.name();
        match self {
            Self::Soldier => format!(
                "{name}:\n\nDuty has called me to the ocean. \
                Will you take me there? \
                I can help you defeat your enemies or clear a path through the trees."),
            Self::Physicist => format!(
                "{name}:\n\nMy studies necessitate that I visit the ocean. \
                Will you take me? \
                If you take me on your boat I will let you borrow my experimental teleportation device."),
            Self::Beast => format!(
                "{name}:\n\n\
                One of those...things...bit me, but jokes on them because it didn't seem to work. \
                Take me to the ocean? \
                I can scare away other beasts that get in your way."),
            Self::Ghost => format!(
                "{name}:\n\n\
                Not all ghosts are scary. \
                I just want to go to the ocean. Will you take me? \
                I can briefly make you incorporeal."),
            Self::Surgeon => format!(
                "{name}:\n\n\
                My skills are needed ad the ocean. \
                Take me there? \
                I can heal you if you get injured."),
            Self::Thief => format!(
                "{name}:\n\n\
                I'm trying to escape to the ocean. \
                Will you help me get there? \
                With me you can sneak past your enemies."),
            Self::Surveyor => format!(
                "{name}:\n\n\
                I wish to go to the ocean to map the coastline. \
                Can I travel on your boat? \
                I'll let you borrow my telescope."),
        }
    }
}
