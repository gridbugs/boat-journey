use crate::{ActionError, Config, GameControlFlow, Input};
use gridbugs::direction::CardinalDirection;
use rand::Rng;
use serde::{Deserialize, Serialize};
use std::time::Duration;

pub struct Game {
    inner_game: crate::Game,
}

#[derive(Serialize, Deserialize)]
pub struct RunningGame {
    game: crate::Game,
}

impl RunningGame {
    pub fn new(game: Game, running: Running) -> Self {
        let _ = running;
        Self {
            game: game.inner_game,
        }
    }

    pub fn into_game(self) -> (Game, Running) {
        (
            Game {
                inner_game: self.game,
            },
            Running(Private),
        )
    }
}

#[derive(Debug)]
struct Private;

#[derive(Debug)]
pub struct Running(Private);
#[derive(Debug)]
pub struct GameOver(Private);

#[derive(Debug)]
pub enum Witness {
    Running(Running),
    GameOver(GameOver),
    Win,
}

impl Witness {
    fn running(private: Private) -> Self {
        Self::Running(Running(private))
    }
}

pub enum ControlInput {
    Walk(CardinalDirection),
    Wait,
}

pub fn new_game<R: Rng>(config: &Config, base_rng: &mut R) -> (Game, Running) {
    let g = Game {
        inner_game: crate::Game::new(config, base_rng),
    };
    (g, Running(Private))
}

impl Running {
    pub fn new_panics() -> Self {
        panic!("this constructor is meant for temporary use during debugging to get the code to compile")
    }

    pub fn into_witness(self) -> Witness {
        Witness::Running(self)
    }

    pub fn tick(self, game: &mut Game, since_last_tick: Duration, config: &Config) -> Witness {
        let Self(private) = self;
        game.witness_handle_tick(since_last_tick, config, private)
    }

    pub fn walk(
        self,
        game: &mut Game,
        direction: CardinalDirection,
        config: &Config,
    ) -> (Witness, Result<(), ActionError>) {
        let Self(private) = self;
        game.witness_handle_input(Input::Walk(direction), config, private)
    }

    pub fn wait(self, game: &mut Game, config: &Config) -> (Witness, Result<(), ActionError>) {
        let Self(private) = self;
        game.witness_handle_input(Input::Wait, config, private)
    }
}

impl GameOver {
    pub fn tick(self, game: &mut Game, since_last_tick: Duration, config: &Config) -> GameOver {
        let Self(private) = self;
        match game.witness_handle_tick(since_last_tick, config, private) {
            Witness::GameOver(game_over) => game_over,
            other => panic!("unexpected witness: {:?}", other),
        }
    }
}

impl Game {
    fn witness_handle_input(
        &mut self,
        input: Input,
        config: &Config,
        private: Private,
    ) -> (Witness, Result<(), ActionError>) {
        match self.inner_game.handle_input(input, config) {
            Err(e) => (Witness::running(private), Err(e)),
            Ok(None) => (Witness::running(private), Ok(())),
            Ok(Some(GameControlFlow::GameOver)) => (Witness::GameOver(GameOver(private)), Ok(())),
            Ok(Some(other)) => panic!("unhandled control flow {:?}", other),
        }
    }

    fn witness_handle_tick(
        &mut self,
        since_last_tick: Duration,
        config: &Config,
        private: Private,
    ) -> Witness {
        match self.inner_game.handle_tick(since_last_tick, config) {
            None => Witness::running(private),
            Some(GameControlFlow::GameOver) => Witness::GameOver(GameOver(private)),
            Some(GameControlFlow::Win) => Witness::Win,
        }
    }

    pub fn inner_ref(&self) -> &crate::Game {
        &self.inner_game
    }

    pub fn into_running_game(self, running: Running) -> RunningGame {
        RunningGame::new(self, running)
    }
}
