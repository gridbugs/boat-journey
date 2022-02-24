use crate::{player, ActionError, Config, ExternalEvent, GameControlFlow, Input};
use direction::CardinalDirection;
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GameOverType {
    Adrift,
    Dead,
}

#[derive(Debug)]
struct Private;

#[derive(Debug)]
pub struct Running(Private);
#[derive(Debug)]
pub struct Upgrade(Private);
#[derive(Debug)]
pub struct GetRangedWeapon(Private);
#[derive(Debug)]
pub struct GetMeleeWeapon(Private);
#[derive(Debug)]
pub struct FireWeapon {
    private: Private,
    slot: player::RangedWeaponSlot,
}
#[derive(Debug)]
pub struct UnlockMap(Private);

#[derive(Debug)]
pub struct GameOver {
    private: Private,
    typ: GameOverType,
}

#[derive(Debug)]
pub enum Witness {
    Running(Running),
    Upgrade(Upgrade),
    GetRangedWeapon(GetRangedWeapon),
    GetMeleeWeapon(GetMeleeWeapon),
    FireWeapon(FireWeapon),
    GameOver(GameOver),
    UnlockMap(UnlockMap),
    Win,
}

impl Witness {
    fn running(private: Private) -> Self {
        Self::Running(Running(private))
    }
    fn upgrade(private: Private) -> Self {
        Self::Upgrade(Upgrade(private))
    }
    fn unlock_map(private: Private) -> Self {
        Self::UnlockMap(UnlockMap(private))
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

    pub fn get(self, game: &Game) -> (Witness, Result<(), ActionError>) {
        if let Some(weapon) = game.inner_ref().weapon_under_player() {
            if weapon.is_ranged() {
                let Self(private) = self;
                return (Witness::GetRangedWeapon(GetRangedWeapon(private)), Ok(()));
            }
            if weapon.is_melee() {
                let Self(private) = self;
                return (Witness::GetMeleeWeapon(GetMeleeWeapon(private)), Ok(()));
            }
        }
        (self.into_witness(), Err(ActionError::NoItemToGet))
    }

    pub fn fire_weapon(
        self,
        game: &Game,
        slot: player::RangedWeaponSlot,
    ) -> (Witness, Result<(), ActionError>) {
        if let Some(weapon) = game.inner_game.player().weapon_in_slot(slot) {
            if weapon.ammo.unwrap().current == 0 {
                return (
                    self.into_witness(),
                    Err(ActionError::WeaponOutOfAmmo(weapon.name)),
                );
            } else {
                let Self(private) = self;
                return (Witness::FireWeapon(FireWeapon { private, slot }), Ok(()));
            }
        }
        (self.into_witness(), Err(ActionError::NoWeaponInSlot(slot)))
    }
}

impl Upgrade {
    pub fn commit(
        self,
        game: &mut Game,
        upgrade: player::Upgrade,
        config: &Config,
    ) -> (Witness, Result<(), ActionError>) {
        let Self(private) = self;
        let input = Input::Upgrade(upgrade);
        game.witness_handle_input(input, config, private)
    }

    pub fn cancel(self) -> Witness {
        let Self(private) = self;
        Witness::running(private)
    }
}

impl GetRangedWeapon {
    pub fn commit(
        self,
        game: &mut Game,
        slot: player::RangedWeaponSlot,
        config: &Config,
    ) -> Witness {
        let Self(private) = self;
        let input = Input::EquipRangedWeapon(slot);
        let (witness, result) = game.witness_handle_input(input, config, private);
        let _ = result.unwrap();
        witness
    }

    pub fn cancel(self) -> Witness {
        let Self(private) = self;
        Witness::running(private)
    }
}

impl GetMeleeWeapon {
    pub fn commit(self, game: &mut Game, config: &Config) -> Witness {
        let Self(private) = self;
        let input = Input::EquipMeleeWeapon;
        let (witness, result) = game.witness_handle_input(input, config, private);
        let _ = result.unwrap();
        witness
    }

    pub fn cancel(self) -> Witness {
        let Self(private) = self;
        Witness::running(private)
    }
}

impl FireWeapon {
    pub fn slot(&self) -> player::RangedWeaponSlot {
        self.slot
    }

    pub fn commit(self, game: &mut Game, direction: CardinalDirection, config: &Config) -> Witness {
        let Self { private, slot } = self;
        let input = Input::Fire { direction, slot };
        let (witness, result) = game.witness_handle_input(input, config, private);
        let _ = result.unwrap();
        witness
    }

    pub fn cancel(self) -> Witness {
        let Self { private, .. } = self;
        Witness::running(private)
    }
}

impl GameOver {
    pub fn typ(&self) -> GameOverType {
        self.typ
    }

    pub fn tick(self, game: &mut Game, since_last_tick: Duration, config: &Config) -> GameOver {
        let Self { private, .. } = self;
        match game.witness_handle_tick(since_last_tick, config, private) {
            Witness::GameOver(game_over) => game_over,
            other => panic!("unexpected witness: {:?}", other),
        }
    }
}

impl UnlockMap {
    pub fn commit(self, game: &mut Game, config: &Config) -> Witness {
        let Self(private) = self;
        let (witness, result) = game.witness_handle_input(Input::UnlockMap, config, private);
        let _ = result.unwrap();
        witness
    }

    pub fn cancel(self) -> Witness {
        let Self(private) = self;
        Witness::running(private)
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
            Ok(Some(GameControlFlow::Upgrade)) => (Witness::upgrade(private), Ok(())),
            Ok(Some(GameControlFlow::GameOver)) => {
                let game_over = if self.inner_game.is_adrift() {
                    GameOver {
                        typ: GameOverType::Adrift,
                        private,
                    }
                } else {
                    GameOver {
                        typ: GameOverType::Dead,
                        private,
                    }
                };
                (Witness::GameOver(game_over), Ok(()))
            }
            Ok(Some(GameControlFlow::LevelChange)) => (Witness::running(private), Ok(())),
            Ok(Some(GameControlFlow::UnlockMap)) => (Witness::unlock_map(private), Ok(())),
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
            Some(GameControlFlow::Upgrade) => Witness::upgrade(private),
            Some(GameControlFlow::GameOver) => {
                let game_over = if self.inner_game.is_adrift() {
                    GameOver {
                        typ: GameOverType::Adrift,
                        private,
                    }
                } else {
                    GameOver {
                        typ: GameOverType::Dead,
                        private,
                    }
                };
                Witness::GameOver(game_over)
            }
            Some(GameControlFlow::LevelChange) => Witness::running(private),
            Some(GameControlFlow::UnlockMap) => Witness::unlock_map(private),
            Some(GameControlFlow::Win) => Witness::Win,
        }
    }

    pub fn inner_ref(&self) -> &crate::Game {
        &self.inner_game
    }

    pub fn into_running_game(self, running: Running) -> RunningGame {
        RunningGame::new(self, running)
    }

    pub fn npc_turn(&mut self) {
        self.inner_game.handle_npc_turn()
    }

    pub fn events(&mut self) -> impl '_ + Iterator<Item = ExternalEvent> {
        self.inner_game.events()
    }

    pub fn player(&self) -> &player::Player {
        self.inner_game.player()
    }
}
