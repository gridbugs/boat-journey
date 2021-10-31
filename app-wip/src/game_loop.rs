use crate::{
    controls::{AppInput, Controls},
    game,
    stars::Stars,
    text,
};
use chargrid::{border::BorderStyle, control_flow::*, input::*, menu, prelude::*};
use general_storage_static::{format, StaticStorage};
use orbital_decay_game::{
    player,
    witness::{self, Game, RunningGame, Witness},
    Config,
};
use rand::{Rng, SeedableRng};
use rand_isaac::Isaac64Rng;
use serde::{Deserialize, Serialize};

const STORAGE_FORMAT: format::Bincode = format::Bincode;

pub enum InitialRngSeed {
    U64(u64),
    Random,
}

struct RngSeedSource {
    next_seed: u64,
    seed_rng: Isaac64Rng,
}

impl RngSeedSource {
    fn new(initial_rng_seed: InitialRngSeed) -> Self {
        let mut seed_rng = Isaac64Rng::from_entropy();
        let next_seed = match initial_rng_seed {
            InitialRngSeed::U64(seed) => seed,
            InitialRngSeed::Random => seed_rng.gen(),
        };
        Self {
            next_seed,
            seed_rng,
        }
    }

    fn next_seed(&mut self) -> u64 {
        let seed = self.next_seed;
        self.next_seed = self.seed_rng.gen();
        #[cfg(feature = "print_stdout")]
        println!("RNG Seed: {}", seed);
        #[cfg(feature = "print_log")]
        log::info!("RNG Seed: {}", seed);
        seed
    }
}

pub struct SaveGameStorage {
    pub handle: StaticStorage,
    pub key: String,
}

impl SaveGameStorage {
    fn save(&mut self, instance: &GameInstanceStorable) {
        let result = self.handle.store(&self.key, &instance, STORAGE_FORMAT);
        if let Err(e) = result {
            use general_storage_static::{StoreError, StoreRawError};
            match e {
                StoreError::FormatError(e) => log::error!("Failed to format save file: {}", e),
                StoreError::Raw(e) => match e {
                    StoreRawError::IoError(e) => {
                        log::error!("Error while writing save data: {}", e)
                    }
                },
            }
        }
    }

    fn load(&self) -> Option<GameInstanceStorable> {
        let result = self
            .handle
            .load::<_, GameInstanceStorable, _>(&self.key, STORAGE_FORMAT);
        match result {
            Err(e) => {
                use general_storage_static::{LoadError, LoadRawError};
                match e {
                    LoadError::FormatError(e) => log::error!("Failed to parse save file: {}", e),
                    LoadError::Raw(e) => match e {
                        LoadRawError::IoError(e) => {
                            log::error!("Error while reading save data: {}", e)
                        }
                        LoadRawError::NoSuchKey => (),
                    },
                }
                None
            }
            Ok(instance) => Some(instance),
        }
    }

    fn clear(&mut self) {
        if self.handle.exists(&self.key) {
            if let Err(e) = self.handle.remove(&self.key) {
                use general_storage_static::RemoveError;
                match e {
                    RemoveError::IoError(e) => {
                        log::error!("Error while removing data: {}", e)
                    }
                    RemoveError::NoSuchKey => (),
                }
            }
        }
    }
}

struct GameInstance {
    game: Game,
    stars: Stars,
}

impl GameInstance {
    fn into_storable(self, running: witness::Running) -> GameInstanceStorable {
        let Self { game, stars } = self;
        let running_game = game.into_running_game(running);
        GameInstanceStorable {
            running_game,
            stars,
        }
    }
}

#[derive(Serialize, Deserialize)]
struct GameInstanceStorable {
    running_game: RunningGame,
    stars: Stars,
}

impl GameInstanceStorable {
    fn into_game_instance(self) -> (GameInstance, witness::Running) {
        let Self {
            running_game,
            stars,
        } = self;
        let (game, running) = running_game.into_game();
        (GameInstance { game, stars }, running)
    }
}

pub struct GameLoopData {
    instance: Option<GameInstance>,
    controls: Controls,
    config: Config,
    save_game_storage: SaveGameStorage,
    rng_seed_source: RngSeedSource,
}

impl GameLoopData {
    pub fn new(
        config: Config,
        save_game_storage: SaveGameStorage,
        initial_rng_seed: InitialRngSeed,
    ) -> (Self, GameLoopState) {
        let (instance, state) = match save_game_storage.load() {
            Some(instance) => {
                let (instance, running) = instance.into_game_instance();
                (
                    Some(instance),
                    GameLoopState::Playing(running.into_witness()),
                )
            }
            None => (None, GameLoopState::MainMenu),
        };
        let controls = Controls::default();
        (
            Self {
                instance,
                controls,
                config,
                save_game_storage,
                rng_seed_source: RngSeedSource::new(initial_rng_seed),
            },
            state,
        )
    }

    fn save_instance(&mut self, running: witness::Running) -> witness::Running {
        let instance = self.instance.take().unwrap().into_storable(running);
        self.save_game_storage.save(&instance);
        let (instance, running) = instance.into_game_instance();
        self.instance = Some(instance);
        running
    }

    fn clear_saved_game(&mut self) {
        self.save_game_storage.clear();
    }

    fn new_game(&mut self) -> witness::Running {
        let mut rng = Isaac64Rng::seed_from_u64(self.rng_seed_source.next_seed());
        let (game, running) = witness::new_game(&self.config, &mut rng);
        let stars = Stars::new(&mut rng);
        self.instance = Some(GameInstance { game, stars });
        running
    }

    fn render(&self, ctx: Ctx, fb: &mut FrameBuffer) {
        let instance = self.instance.as_ref().unwrap();
        instance
            .stars
            .render_with_visibility(instance.game.inner_ref().visibility_grid(), ctx, fb);
        game::render_game(instance.game.inner_ref(), ctx, fb);
    }

    fn has_game_ever_been_won(&self) -> bool {
        // TODO
        true
    }
    fn update(&mut self, event: Event, running: witness::Running) -> Witness {
        let instance = self.instance.as_mut().unwrap();
        match event {
            Event::Input(Input::Keyboard(keyboard_input)) => {
                if let Some(app_input) = self.controls.get(keyboard_input) {
                    let (witness, action_result) = match app_input {
                        AppInput::Move(direction) => {
                            running.walk(&mut instance.game, direction, &self.config)
                        }
                        AppInput::Wait => running.wait(&mut instance.game, &self.config),
                        AppInput::Examine | AppInput::Aim(_) | AppInput::Get => {
                            println!("todo");
                            (Witness::Running(running), Ok(()))
                        }
                    };
                    if let Err(action_error) = action_result {
                        println!("action error: {:?}", action_error);
                    }
                    witness
                } else {
                    Witness::Running(running)
                }
            }
            Event::Tick(since_previous) => {
                running.tick(&mut instance.game, since_previous, &self.config)
            }
            _ => Witness::Running(running),
        }
    }
}

struct GameInstanceComponent(Option<witness::Running>);

impl GameInstanceComponent {
    fn new(running: witness::Running) -> Self {
        Self(Some(running))
    }
}

pub enum GameLoopState {
    Paused(witness::Running),
    Playing(Witness),
    MainMenu,
}

impl Component for GameInstanceComponent {
    type Output = GameLoopState;
    type State = GameLoopData;
    fn render(&self, state: &Self::State, ctx: Ctx, fb: &mut FrameBuffer) {
        state.render(ctx, fb);
    }

    fn update(&mut self, state: &mut Self::State, _ctx: Ctx, event: Event) -> Self::Output {
        let running = self.0.take().unwrap();
        if event.is_escape() {
            GameLoopState::Paused(running)
        } else {
            GameLoopState::Playing(state.update(event, running))
        }
    }

    fn size(&self, _state: &Self::State, ctx: Ctx) -> Size {
        ctx.bounding_box.size()
    }
}

fn upgrade_identifier(upgrade: player::Upgrade) -> String {
    let name = match upgrade.typ {
        player::UpgradeType::Toughness => "Toughness",
        player::UpgradeType::Accuracy => "Accuracy",
        player::UpgradeType::Endurance => "Endurance",
    };
    let level = match upgrade.level {
        player::UpgradeLevel::Level1 => "1",
        player::UpgradeLevel::Level2 => "2",
    };
    let price = upgrade.level.cost();
    format!("{} {} (${})", name, level, price)
}

fn upgrade_menu(
    upgrades: Vec<player::Upgrade>,
) -> CF<impl Component<State = GameLoopData, Output = Option<player::Upgrade>>> {
    use menu::builder::*;
    let mut builder = menu_builder();
    for upgrade in upgrades {
        builder = builder.add_item(item(
            upgrade,
            identifier::simple(upgrade_identifier(upgrade).as_str()),
        ));
    }
    builder
        .build_cf()
        .border(BorderStyle::default())
        .centre()
        .overlay(
            render_state(|state: &GameLoopData, ctx, fb| state.render(ctx, fb)),
            chargrid::core::TintDim(63),
            10,
        )
}

fn upgrade_component(
    upgrade_witness: witness::Upgrade,
) -> CF<impl Component<State = GameLoopData, Output = Option<Witness>>> {
    on_state_then(|state: &mut GameLoopData| {
        let instance = state.instance.as_ref().unwrap();
        let upgrades = instance.game.inner_ref().available_upgrades();
        upgrade_menu(upgrades).catch_escape().and_then(|result| {
            on_state(move |state: &mut GameLoopData| {
                let (witness, result) = match result {
                    Err(Escape) => upgrade_witness.cancel(),
                    Ok(upgrade) => {
                        let instance = state.instance.as_mut().unwrap();
                        upgrade_witness.upgrade(&mut instance.game, upgrade, &state.config)
                    }
                };
                if let Err(upgrade_error) = result {
                    println!("upgrade error: {:?}", upgrade_error);
                }
                witness
            })
        })
    })
}

#[derive(Clone)]
enum MainMenuEntry {
    NewGame,
    Options,
    Help,
    Prologue,
    Epilogue,
    Quit,
}

fn main_menu() -> CF<impl Component<State = GameLoopData, Output = Option<MainMenuEntry>>> {
    on_state_then(|state: &mut GameLoopData| {
        use menu::builder::*;
        use MainMenuEntry::*;
        let mut builder = menu_builder();
        let mut add_item = |entry, name, ch: char| {
            builder.add_item_mut(
                item(entry, identifier::simple(&format!("({}) {}", ch, name))).add_hotkey_char(ch),
            );
        };
        add_item(NewGame, "New Game", 'n');
        add_item(Options, "Options", 'o');
        add_item(Help, "Help", 'h');
        add_item(Prologue, "Prologue", 'p');
        if state.has_game_ever_been_won() {
            add_item(Epilogue, "Epilogue", 'e');
        }
        add_item(Quit, "Quit", 'q');
        builder.build_cf().border(BorderStyle::default()).centre()
    })
}

enum MainMenuOutput {
    NewGame { new_running: witness::Running },
    MainMenu,
    Quit,
}

fn prologue() -> CF<impl Component<State = GameLoopData, Output = Option<()>>> {
    text::prologue().ignore_state().press_any_key()
}

fn epilogue() -> CF<impl Component<State = GameLoopData, Output = Option<()>>> {
    unit().press_any_key()
}

fn main_menu_loop() -> CF<impl Component<State = GameLoopData, Output = Option<MainMenuOutput>>> {
    use MainMenuEntry::*;
    either!(Ei = A | B | C | D | E | F);
    main_menu().and_then(|entry| match entry {
        NewGame => Ei::A(on_state(|state: &mut GameLoopData| {
            MainMenuOutput::NewGame {
                new_running: state.new_game(),
            }
        })),
        Options => Ei::B(never()),
        Help => Ei::C(never()),
        Prologue => Ei::D(prologue().map(|()| MainMenuOutput::MainMenu)),
        Epilogue => Ei::E(epilogue().map(|()| MainMenuOutput::MainMenu)),
        Quit => Ei::F(val_once(MainMenuOutput::Quit)),
    })
}

#[derive(Clone)]
enum PauseMenuEntry {
    Resume,
    SaveQuit,
    Save,
    NewGame,
    Options,
    Help,
    Prologue,
    Epilogue,
    Clear,
}

fn pause_menu() -> CF<impl Component<State = GameLoopData, Output = Option<PauseMenuEntry>>> {
    on_state_then(|state: &mut GameLoopData| {
        use menu::builder::*;
        use PauseMenuEntry::*;
        let mut builder = menu_builder();
        let mut add_item = |entry, name, ch: char| {
            builder.add_item_mut(
                item(entry, identifier::simple(&format!("({}) {}", ch, name))).add_hotkey_char(ch),
            );
        };
        add_item(Resume, "Resume", 'r');
        add_item(SaveQuit, "Save and Quit", 'q');
        add_item(Save, "Save", 's');
        add_item(NewGame, "New Game", 'n');
        add_item(Options, "Options", 'o');
        add_item(Help, "Help", 'h');
        add_item(Prologue, "Prologue", 'p');
        if state.has_game_ever_been_won() {
            add_item(Epilogue, "Epilogue", 'e');
        }
        add_item(Clear, "Clear", 'c');
        builder
            .build_cf()
            .border(BorderStyle::default())
            .centre()
            .overlay(
                render_state(|state: &GameLoopData, ctx, fb| state.render(ctx, fb)),
                chargrid::core::TintDim(63),
                10,
            )
    })
}

enum PauseOutput {
    Continue { running: witness::Running },
    Restart { new_running: witness::Running },
    MainMenu,
    PauseMenu { running: witness::Running },
    Quit,
}

fn pause(
    running: witness::Running,
) -> CF<impl Component<State = GameLoopData, Output = Option<PauseOutput>>> {
    use PauseMenuEntry::*;
    either!(Ei = A | B | C | D | E | F | G | H | I);
    pause_menu()
        .catch_escape()
        .and_then(|entry_or_escape| match entry_or_escape {
            Ok(entry) => match entry {
                Resume => Ei::A(val_once(PauseOutput::Continue { running })),
                SaveQuit => Ei::B(on_state(|state: &mut GameLoopData| {
                    state.save_instance(running);
                    PauseOutput::Quit
                })),
                Save => Ei::C(on_state(|state: &mut GameLoopData| PauseOutput::Continue {
                    running: state.save_instance(running),
                })),
                NewGame => Ei::D(on_state(|state: &mut GameLoopData| PauseOutput::Restart {
                    new_running: state.new_game(),
                })),
                Options => Ei::E(never()),
                Help => Ei::F(never()),
                Prologue => Ei::G(prologue().map(|()| PauseOutput::PauseMenu { running })),
                Epilogue => Ei::H(epilogue().map(|()| PauseOutput::PauseMenu { running })),
                Clear => Ei::I(on_state(|state: &mut GameLoopData| {
                    state.clear_saved_game();
                    PauseOutput::MainMenu
                })),
            },
            Err(Escape) => Ei::A(val_once(PauseOutput::Continue { running })),
        })
}

fn game_instance_component(
    running: witness::Running,
) -> CF<impl Component<State = GameLoopData, Output = Option<GameLoopState>>> {
    cf(GameInstanceComponent::new(running)).some()
}

pub enum GameExitReason {
    GameOver,
    Quit,
}

pub fn game_loop_component(
    initial_state: GameLoopState,
) -> CF<impl Component<State = GameLoopData, Output = Option<GameExitReason>>> {
    use GameLoopState::*;
    loop_(initial_state, |state| {
        either!(Ei = A | B | C | D | E);
        match state {
            Playing(witness) => match witness {
                Witness::Running(running) => Ei::A(game_instance_component(running).continue_()),
                Witness::Upgrade(upgrade) => {
                    Ei::B(upgrade_component(upgrade).map(Playing).continue_())
                }
                Witness::GameOver => Ei::C(val_once(GameExitReason::GameOver).break_()),
            },
            Paused(running) => Ei::D(pause(running).map(|pause_output| match pause_output {
                PauseOutput::Continue { running } => {
                    LoopControl::Continue(Playing(running.into_witness()))
                }
                PauseOutput::Restart { new_running } => {
                    LoopControl::Continue(Playing(new_running.into_witness()))
                }
                PauseOutput::MainMenu => LoopControl::Continue(MainMenu),
                PauseOutput::PauseMenu { running } => LoopControl::Continue(Paused(running)),
                PauseOutput::Quit => LoopControl::Break(GameExitReason::Quit),
            })),
            MainMenu => Ei::E(
                main_menu_loop().map(|main_menu_output| match main_menu_output {
                    MainMenuOutput::NewGame { new_running } => {
                        LoopControl::Continue(Playing(new_running.into_witness()))
                    }
                    MainMenuOutput::MainMenu => LoopControl::Continue(MainMenu),
                    MainMenuOutput::Quit => LoopControl::Break(GameExitReason::Quit),
                }),
            ),
        }
    })
}
