pub use gridbugs::{
    direction::CardinalDirection,
    entity_table::ComponentTable,
    grid_2d::{Coord, Grid, Size},
    shadowcast::Context as ShadowcastContext,
};
use rand::{seq::SliceRandom, Rng, SeedableRng};
use rand_isaac::Isaac64Rng;
use serde::{Deserialize, Serialize};
use std::time::Duration;

pub mod witness;
mod world;

pub use gridbugs::entity_table::Entity;
use world::World;

pub const MAP_SIZE: Size = Size::new_u16(20, 14);

#[derive(Debug, Clone, Copy)]
pub struct Omniscient;

#[derive(Debug, Clone, Copy)]
pub struct Config {
    pub omniscient: Option<Omniscient>,
    pub demo: bool,
    pub debug: bool,
}
impl Config {
    pub const OMNISCIENT: Option<Omniscient> = Some(Omniscient);
}
impl Default for Config {
    fn default() -> Self {
        Self {
            omniscient: None,
            demo: false,
            debug: false,
        }
    }
}

#[derive(Debug)]
pub enum GameControlFlow {
    GameOver,
    Win,
}

#[derive(Clone, Copy, Debug)]
pub enum Input {
    Walk(CardinalDirection),
    Wait,
}

#[derive(Serialize, Deserialize)]
pub struct Game {
    world: World,
    rng: Isaac64Rng,
}

pub enum ActionError {}

impl Game {
    pub fn new<R: Rng>(config: &Config, base_rng: &mut R) -> Self {
        let mut rng = Isaac64Rng::seed_from_u64(base_rng.gen());
        let world = World::new(MAP_SIZE);
        let mut game = Self { rng, world };
        game
    }

    #[must_use]
    pub fn handle_tick(
        &mut self,
        since_last_tick: Duration,
        config: &Config,
    ) -> Option<GameControlFlow> {
        None
    }

    #[must_use]
    pub fn handle_input(
        &mut self,
        input: Input,
        config: &Config,
    ) -> Result<Option<GameControlFlow>, ActionError> {
        Ok(None)
    }
}
