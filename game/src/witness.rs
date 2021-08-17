use crate::{player, ActionError, Config, Game, GameControlFlow, Input};
use direction::CardinalDirection;
use rand::Rng;
use std::time::Duration;

struct Private;

pub struct Running(Private);
pub struct Upgrade(Private);

pub enum Witness {
    Running(Running),
    Upgrade(Upgrade),
    GameOver,
}

impl Witness {
    fn running(private: Private) -> Self {
        Self::Running(Running(private))
    }
    fn upgrade(private: Private) -> Self {
        Self::Upgrade(Upgrade(private))
    }
}

pub enum ControlInput {
    Walk(CardinalDirection),
    Wait,
}

impl Game {
    pub fn witness_new<R: Rng>(config: &Config, base_rng: &mut R) -> (Self, Running) {
        let s = Self::new(config, base_rng);
        (s, Running(Private))
    }

    pub fn witness_tick(
        &mut self,
        since_last_tick: Duration,
        config: &Config,
        Running(private): Running,
    ) -> Witness {
        use GameControlFlow::*;
        match self.handle_tick(since_last_tick, config) {
            None => Witness::running(private),
            Some(Upgrade) => Witness::upgrade(private),
            Some(GameOver) => Witness::GameOver,
            Some(_) => todo!(),
        }
    }

    pub fn witness_walk(
        &mut self,
        direction: CardinalDirection,
        config: &Config,
        Running(private): Running,
    ) -> (Witness, Result<(), ActionError>) {
        self.witness_handle_input(Input::Walk(direction), config, private)
    }

    pub fn witness_wait(
        &mut self,
        config: &Config,
        Running(private): Running,
    ) -> (Witness, Result<(), ActionError>) {
        self.witness_handle_input(Input::Wait, config, private)
    }

    pub fn witness_upgrade(
        &mut self,
        upgrade: player::Upgrade,
        config: &Config,
        Upgrade(private): Upgrade,
    ) -> (Witness, Result<(), ActionError>) {
        let input = Input::Upgrade(upgrade);
        self.witness_handle_input(input, config, private)
    }

    pub fn witness_upgrade_cancel(
        &mut self,
        config: &Config,
        Upgrade(private): Upgrade,
    ) -> (Witness, Result<(), ActionError>) {
        (Witness::running(private), Ok(()))
    }

    fn witness_handle_input(
        &mut self,
        input: Input,
        config: &Config,
        private: Private,
    ) -> (Witness, Result<(), ActionError>) {
        use GameControlFlow::*;
        match self.handle_input(input, config) {
            Err(e) => (Witness::running(private), Err(e)),
            Ok(None) => (Witness::running(private), Ok(())),
            Ok(Some(Upgrade)) => (Witness::upgrade(private), Ok(())),
            Ok(Some(GameOver)) => (Witness::GameOver, Ok(())),
            Ok(Some(_)) => todo!(),
        }
    }
}
