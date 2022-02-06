use crate::audio::{AppAudioPlayer, Audio, AudioState};
use crate::{
    colours,
    controls::{AppInput, Controls},
    game_instance::{GameInstance, GameInstanceStorable},
    menu_background::MenuBackground,
    text,
};
use chargrid::{
    border::BorderStyle, control_flow::boxed::*, input::*, menu, menu::Menu, pad_by::Padding,
    prelude::*, text::StyledString,
};
use general_storage_static::{format, StaticStorage};
use orbital_decay_game::{
    player,
    witness::{self, Witness},
    Config as GameConfig, ExternalEvent, Music,
};
use rand::{Rng, SeedableRng};
use rand_isaac::Isaac64Rng;
use serde::{Deserialize, Serialize};

fn game_music_to_audio(music: Music) -> Audio {
    match music {
        Music::Gameplay0 => Audio::Gameplay0,
        Music::Gameplay1 => Audio::Gameplay1,
        Music::Gameplay2 => Audio::Gameplay2,
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
struct Config {
    music_volume: f32,
    sfx_volume: f32,
    won: bool,
    first_run: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            music_volume: 0.2,
            sfx_volume: 0.5,
            won: true,
            first_run: true,
        }
    }
}

/// An interactive, renderable process yielding a value of type `T`
pub type CF<T> = BoxedCF<Option<T>, GameLoopData>;

const MENU_BACKGROUND: Rgba32 = colours::SPACE_BACKGROUND.saturating_scalar_mul_div(2, 3);
const MENU_FADE_SPEC: menu::identifier::fade_spec::FadeSpec = {
    use menu::identifier::fade_spec::*;
    FadeSpec {
        on_select: Fade {
            to: To {
                rgba32: Layers {
                    foreground: colours::WALL_TOP,
                    background: colours::STRIPE,
                },
                bold: true,
                underline: false,
            },
            from: From::current(),
            durations: Layers {
                foreground: Duration::from_millis(128),
                background: Duration::from_millis(128),
            },
        },
        on_deselect: Fade {
            to: To {
                rgba32: Layers {
                    foreground: colours::STRIPE,
                    background: MENU_BACKGROUND,
                },
                bold: false,
                underline: false,
            },
            from: From::current(),
            durations: Layers {
                foreground: Duration::from_millis(128),
                background: Duration::from_millis(128),
            },
        },
    }
};

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

pub struct AppStorage {
    pub handle: StaticStorage,
    pub save_game_key: String,
    pub config_key: String,
}

impl AppStorage {
    const SAVE_GAME_STORAGE_FORMAT: format::Bincode = format::Bincode;
    const CONFIG_STORAGE_FORMAT: format::Json = format::Json;

    fn save_game(&mut self, instance: &GameInstanceStorable) {
        let result = self.handle.store(
            &self.save_game_key,
            &instance,
            Self::SAVE_GAME_STORAGE_FORMAT,
        );
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

    fn load_game(&self) -> Option<GameInstanceStorable> {
        let result = self.handle.load::<_, GameInstanceStorable, _>(
            &self.save_game_key,
            Self::SAVE_GAME_STORAGE_FORMAT,
        );
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

    fn clear_game(&mut self) {
        if self.handle.exists(&self.save_game_key) {
            if let Err(e) = self.handle.remove(&self.save_game_key) {
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

    fn save_config(&mut self, config: &Config) {
        let result = self
            .handle
            .store(&self.config_key, &config, Self::CONFIG_STORAGE_FORMAT);
        if let Err(e) = result {
            use general_storage_static::{StoreError, StoreRawError};
            match e {
                StoreError::FormatError(e) => log::error!("Failed to format config: {}", e),
                StoreError::Raw(e) => match e {
                    StoreRawError::IoError(e) => {
                        log::error!("Error while writing config: {}", e)
                    }
                },
            }
        }
    }

    fn load_config(&self) -> Option<Config> {
        let result = self
            .handle
            .load::<_, Config, _>(&self.config_key, Self::CONFIG_STORAGE_FORMAT);
        match result {
            Err(e) => {
                use general_storage_static::{LoadError, LoadRawError};
                match e {
                    LoadError::FormatError(e) => log::error!("Failed to parse config file: {}", e),
                    LoadError::Raw(e) => match e {
                        LoadRawError::IoError(e) => {
                            log::error!("Error while reading config: {}", e)
                        }
                        LoadRawError::NoSuchKey => (),
                    },
                }
                None
            }
            Ok(instance) => Some(instance),
        }
    }
}

fn new_game(
    rng_seed_source: &mut RngSeedSource,
    game_config: &GameConfig,
) -> (GameInstance, witness::Running) {
    let mut rng = Isaac64Rng::seed_from_u64(rng_seed_source.next_seed());
    GameInstance::new(game_config, &mut rng)
}

pub struct GameLoopData {
    instance: Option<GameInstance>,
    controls: Controls,
    game_config: GameConfig,
    storage: AppStorage,
    rng_seed_source: RngSeedSource,
    menu_background: MenuBackground,
    audio_state: AudioState,
    config: Config,
}

impl GameLoopData {
    pub fn new(
        game_config: GameConfig,
        storage: AppStorage,
        initial_rng_seed: InitialRngSeed,
        audio_player: AppAudioPlayer,
        force_new_game: bool,
    ) -> (Self, GameLoopState) {
        let mut rng_seed_source = RngSeedSource::new(initial_rng_seed);
        let (instance, state) = match storage.load_game() {
            Some(instance) => {
                let (instance, running) = instance.into_game_instance();
                (
                    Some(instance),
                    GameLoopState::Playing(running.into_witness()),
                )
            }
            None => {
                if force_new_game {
                    let (instance, running) = new_game(&mut rng_seed_source, &game_config);
                    (
                        Some(instance),
                        GameLoopState::Playing(running.into_witness()),
                    )
                } else {
                    (None, GameLoopState::MainMenu)
                }
            }
        };
        let controls = Controls::default();
        let menu_background = MenuBackground::new(&mut Isaac64Rng::from_entropy());
        let mut audio_state = AudioState::new(audio_player);
        let config = storage.load_config().unwrap_or_default();
        if let Some(instance) = instance.as_ref() {
            if let Some(music) = instance.current_music {
                audio_state.loop_music(game_music_to_audio(music), config.music_volume);
            }
        } else {
            audio_state.loop_music(Audio::Menu, config.music_volume);
        }
        (
            Self {
                instance,
                controls,
                game_config,
                storage,
                rng_seed_source,
                menu_background,
                audio_state,
                config,
            },
            state,
        )
    }

    fn save_instance(&mut self, running: witness::Running) -> witness::Running {
        let instance = self.instance.take().unwrap().into_storable(running);
        self.storage.save_game(&instance);
        let (instance, running) = instance.into_game_instance();
        self.instance = Some(instance);
        running
    }

    fn clear_saved_game(&mut self) {
        self.storage.clear_game();
        self.audio_state
            .loop_music(Audio::Menu, self.config.music_volume);
    }

    fn new_game(&mut self) -> witness::Running {
        let (instance, running) = new_game(&mut self.rng_seed_source, &self.game_config);
        self.instance = Some(instance);
        running
    }

    fn save_config(&mut self) {
        self.storage.save_config(&self.config);
    }

    fn render(&self, ctx: Ctx, fb: &mut FrameBuffer) {
        let instance = self.instance.as_ref().unwrap();
        instance.render(ctx, fb);
    }

    fn update(&mut self, event: Event, running: witness::Running) -> Witness {
        let instance = self.instance.as_mut().unwrap();
        let witness = match event {
            Event::Input(Input::Keyboard(keyboard_input)) => {
                if let Some(app_input) = self.controls.get(keyboard_input) {
                    let (witness, action_result) = match app_input {
                        AppInput::Move(direction) => {
                            running.walk(&mut instance.game, direction, &self.game_config)
                        }
                        AppInput::Wait => running.wait(&mut instance.game, &self.game_config),
                        AppInput::Get => running.get(&instance.game),
                        AppInput::Examine | AppInput::Aim(_) => {
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
                running.tick(&mut instance.game, since_previous, &self.game_config)
            }
            _ => Witness::Running(running),
        };
        self.handle_game_events();
        witness
    }

    fn handle_game_events(&mut self) {
        let instance = self.instance.as_mut().unwrap();
        for event in instance.game.events() {
            match event {
                ExternalEvent::LoopMusic(music) => {
                    instance.current_music = Some(music);
                    self.audio_state
                        .loop_music(game_music_to_audio(music), self.config.music_volume);
                }
                _ => (),
            }
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

struct MenuBackgroundComponent;

impl Component for MenuBackgroundComponent {
    type Output = ();
    type State = GameLoopData;

    fn render(&self, state: &Self::State, ctx: Ctx, fb: &mut FrameBuffer) {
        state.menu_background.render(ctx, fb);
    }

    fn update(&mut self, state: &mut Self::State, _ctx: Ctx, event: Event) -> Self::Output {
        if let Some(duration) = event.tick() {
            state.menu_background.tick(duration);
        }
    }

    fn size(&self, _state: &Self::State, ctx: Ctx) -> Size {
        ctx.bounding_box.size()
    }
}

fn menu_style<T: 'static>(menu: CF<T>) -> CF<T> {
    menu.border(BorderStyle::default())
        .fill(MENU_BACKGROUND)
        .centre()
        .overlay(
            render_state(|state: &GameLoopData, ctx, fb| state.render(ctx, fb)),
            chargrid::core::TintDim(63),
            10,
        )
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

fn upgrade_description(upgrade: &player::Upgrade) -> &'static str {
    use player::{UpgradeLevel::*, UpgradeType::*};
    match upgrade {
        player::Upgrade {
            typ: Toughness,
            level: Level1,
        } => "Toughness 1: Strong Back\nGain a third ranged weapon slot.",
        player::Upgrade {
            typ: Toughness,
            level: Level2,
        } => "Toughness 2: Hardy\nDouble your maximum health.",
        player::Upgrade {
            typ: Accuracy,
            level: Level1,
        } => "Accuracy 1: Careful\nReduce hull pen chance by half.",
        player::Upgrade {
            typ: Accuracy,
            level: Level2,
        } => "Accuracy 2: Kill Shot\nDeal double damage to enemies.",
        player::Upgrade {
            typ: Endurance,
            level: Level1,
        } => "Endurance 1: Sure-Footed\nVacuum pulls you one space each turn instead of two.",
        player::Upgrade {
            typ: Endurance,
            level: Level2,
        } => "Endurance 2: Big Lungs\nDouble your maximum oxygen.",
    }
}

struct UpgradeMenuDecorated {
    menu: Menu<player::Upgrade>,
}
impl UpgradeMenuDecorated {
    const MENU_Y_OFFSET: i32 = 4;
    const TEXT_STYLE: Style = Style::new()
        .with_bold(false)
        .with_foreground(Rgba32::new_grey(255));
    const SIZE: Size = Size::new_u16(33, 13);

    fn text(ctx: Ctx, fb: &mut FrameBuffer, string: String) {
        StyledString {
            string,
            style: Self::TEXT_STYLE,
        }
        .render(&(), ctx, fb);
    }
}
impl Component for UpgradeMenuDecorated {
    type Output = Option<player::Upgrade>;
    type State = GameLoopData;

    fn render(&self, state: &Self::State, ctx: Ctx, fb: &mut FrameBuffer) {
        let instance = state.instance.as_ref().unwrap();
        let balance = instance.game.player().credit;
        Self::text(ctx, fb, "Buy an Upgrade (escape cancels)".to_string());
        Self::text(ctx.add_y(2), fb, format!("Your balance: ${}", balance));
        self.menu.render(&(), ctx.add_y(Self::MENU_Y_OFFSET), fb);
        let description = upgrade_description(self.menu.selected());
        StyledString {
            string: description.to_string(),
            style: Self::TEXT_STYLE,
        }
        .wrap_word()
        .cf()
        .bound_width(Self::SIZE.width())
        .render(&(), ctx.add_y(9), fb);
    }

    fn update(&mut self, _state: &mut Self::State, ctx: Ctx, event: Event) -> Self::Output {
        self.menu
            .update(&mut (), ctx.add_y(Self::MENU_Y_OFFSET), event)
    }

    fn size(&self, _state: &Self::State, _ctx: Ctx) -> Size {
        Self::SIZE
    }
}

fn upgrade_menu() -> CF<player::Upgrade> {
    on_state_then(|state: &mut GameLoopData| {
        let instance = state.instance.as_ref().unwrap();
        let upgrades = instance.game.inner_ref().available_upgrades();
        use menu::builder::*;
        let mut builder = menu_builder();
        for upgrade in upgrades {
            let name = upgrade_identifier(upgrade);
            let identifier = MENU_FADE_SPEC.identifier(move |b| write!(b, "{}", name).unwrap());
            builder = builder.add_item(item(upgrade, identifier));
        }
        let menu = builder.build();
        UpgradeMenuDecorated { menu }
    })
}

fn popup(string: String) -> CF<()> {
    menu_style(
        StyledString {
            string,
            style: Style::new()
                .with_bold(false)
                .with_underline(false)
                .with_foreground(colours::STRIPE),
        }
        .boxed_cf()
        .pad_by(Padding::all(1))
        .press_any_key(),
    )
}

fn upgrade_component(upgrade_witness: witness::Upgrade) -> CF<Witness> {
    menu_style(upgrade_menu())
        .catch_escape()
        .and_then(|result| {
            on_state_then(move |state: &mut GameLoopData| match result {
                Err(Escape) => val_once(upgrade_witness.cancel()),
                Ok(upgrade) => {
                    let instance = state.instance.as_mut().unwrap();
                    if upgrade.level.cost() > instance.game.player().credit {
                        popup("You can't afford that!".to_string())
                            .map_val(|| upgrade_witness.cancel())
                    } else {
                        let (witness, result) = upgrade_witness.upgrade(
                            &mut instance.game,
                            upgrade,
                            &state.game_config,
                        );
                        if let Err(upgrade_error) = result {
                            println!("upgrade error: {:?}", upgrade_error);
                        }
                        val_once(witness)
                    }
                }
            })
        })
}

fn try_upgrade_component(upgrade_witness: witness::Upgrade) -> CF<Witness> {
    on_state_then(move |state: &mut GameLoopData| {
        let instance = state.instance.as_ref().unwrap();
        let upgrades = instance.game.inner_ref().available_upgrades();
        if upgrades.is_empty() {
            popup("No remaining upgrades!".to_string()).map_val(|| upgrade_witness.cancel())
        } else {
            upgrade_component(upgrade_witness)
        }
    })
}

fn try_get_ranged_weapon(witness: witness::GetRangedWeapon) -> CF<Witness> {
    todo!()
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

fn title_decorate<T: 'static>(cf: CF<T>) -> CF<T> {
    let decoration = {
        let style = Style::default().with_foreground(colours::WALL_FRONT);
        chargrid::boxed_many![
            styled_string("Orbital Decay".to_string(), style.with_bold(true))
                .add_offset(Coord { x: 14, y: 24 }),
            styled_string(
                "Programming and art by Stephen Sherratt".to_string(),
                style.with_bold(false)
            )
            .add_offset(Coord { x: 1, y: 57 }),
            styled_string(
                "Music and sound effects by Lily Chen".to_string(),
                style.with_bold(false)
            )
            .add_offset(Coord { x: 1, y: 58 }),
        ]
    };
    cf.add_offset(Coord { x: 14, y: 28 })
        .overlay(decoration, chargrid::core::TintIdentity, 10)
}

fn main_menu() -> CF<MainMenuEntry> {
    on_state_then(|state: &mut GameLoopData| {
        use menu::builder::*;
        use MainMenuEntry::*;
        let mut builder = menu_builder();
        let mut add_item = |entry, name, ch: char| {
            let identifier =
                MENU_FADE_SPEC.identifier(move |b| write!(b, "({}) {}", ch, name).unwrap());
            builder.add_item_mut(item(entry, identifier).add_hotkey_char(ch));
        };
        add_item(NewGame, "New Game", 'n');
        add_item(Options, "Options", 'o');
        add_item(Help, "Help", 'h');
        add_item(Prologue, "Prologue", 'p');
        if state.config.won {
            add_item(Epilogue, "Epilogue", 'e');
        }
        add_item(Quit, "Quit", 'q');
        builder.build_cf()
    })
}

enum MainMenuOutput {
    NewGame { new_running: witness::Running },
    Quit,
}

const MAIN_MENU_TEXT_WIDTH: u32 = 40;

fn main_menu_loop() -> CF<MainMenuOutput> {
    use MainMenuEntry::*;
    title_decorate(main_menu())
        .repeat_unit(move |entry| match entry {
            NewGame => on_state(|state: &mut GameLoopData| MainMenuOutput::NewGame {
                new_running: state.new_game(),
            })
            .break_(),
            Options => title_decorate(options_menu()).continue_(),
            Help => text::help(MAIN_MENU_TEXT_WIDTH).centre().continue_(),
            Prologue => text::prologue(MAIN_MENU_TEXT_WIDTH).centre().continue_(),
            Epilogue => text::epilogue(MAIN_MENU_TEXT_WIDTH).centre().continue_(),
            Quit => val_once(MainMenuOutput::Quit).break_(),
        })
        .bound_width(42)
        .overlay(MenuBackgroundComponent, chargrid::core::TintIdentity, 10)
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

fn pause_menu() -> CF<PauseMenuEntry> {
    on_state_then(|state: &mut GameLoopData| {
        use menu::builder::*;
        use PauseMenuEntry::*;
        let mut builder = menu_builder();
        let mut add_item = |entry, name, ch: char| {
            let identifier =
                MENU_FADE_SPEC.identifier(move |b| write!(b, "({}) {}", ch, name).unwrap());
            builder.add_item_mut(item(entry, identifier).add_hotkey_char(ch));
        };
        add_item(Resume, "Resume", 'r');
        add_item(SaveQuit, "Save and Quit", 'q');
        add_item(Save, "Save", 's');
        add_item(NewGame, "New Game", 'n');
        add_item(Options, "Options", 'o');
        add_item(Help, "Help", 'h');
        add_item(Prologue, "Prologue", 'p');
        if state.config.won {
            add_item(Epilogue, "Epilogue", 'e');
        }
        add_item(Clear, "Clear", 'c');
        builder.build_cf()
    })
}

fn pause_menu_loop(running: witness::Running) -> CF<PauseOutput> {
    use PauseMenuEntry::*;
    let text_width = 64;
    pause_menu()
        .catch_escape()
        .repeat(
            running,
            move |running, entry_or_escape| match entry_or_escape {
                Ok(entry) => match entry {
                    Resume => break_(PauseOutput::ContinueGame { running }),
                    SaveQuit => on_state(|state: &mut GameLoopData| {
                        state.save_instance(running);
                        PauseOutput::Quit
                    })
                    .break_(),
                    Save => on_state(|state: &mut GameLoopData| PauseOutput::ContinueGame {
                        running: state.save_instance(running),
                    })
                    .break_(),
                    NewGame => on_state(|state: &mut GameLoopData| PauseOutput::ContinueGame {
                        running: state.new_game(),
                    })
                    .break_(),
                    Options => options_menu().continue_with(running),
                    Help => text::help(text_width).continue_with(running),
                    Prologue => text::prologue(text_width).continue_with(running),
                    Epilogue => text::epilogue(text_width).continue_with(running),
                    Clear => on_state(|state: &mut GameLoopData| {
                        state.clear_saved_game();
                        PauseOutput::MainMenu
                    })
                    .break_(),
                },
                Err(Escape) => break_(PauseOutput::ContinueGame { running }),
            },
        )
}

enum PauseOutput {
    ContinueGame { running: witness::Running },
    MainMenu,
    Quit,
}

fn pause(running: witness::Running) -> CF<PauseOutput> {
    const PAUSE_MUSIC_VOLUME_MULTIPLIER: f32 = 0.25;
    on_state_then(move |state: &mut GameLoopData| {
        // turn down the music in the pause menu
        state
            .audio_state
            .set_music_volume_multiplier(PAUSE_MUSIC_VOLUME_MULTIPLIER);
        menu_style(pause_menu_loop(running))
    })
    .then_side_effect(|state: &mut GameLoopData| {
        // turn the music back up
        state.audio_state.set_music_volume_multiplier(1.);
    })
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum OptionsMenuEntry {
    MusicVolume,
    SfxVolume,
    Back,
}
struct OptionsMenuComponent {
    menu: Menu<OptionsMenuEntry>,
}
impl Component for OptionsMenuComponent {
    type Output = Option<()>;
    type State = GameLoopData;

    fn render(&self, state: &Self::State, ctx: Ctx, fb: &mut FrameBuffer) {
        self.menu.render(&(), ctx, fb);
        let x_offset = 14;
        let style = Style::default()
            .with_foreground(Rgba32::new_grey(255))
            .with_bold(false);
        StyledString {
            string: format!("< {:.0}% >", state.config.music_volume * 100.),
            style,
        }
        .render(&(), ctx.add_offset(Coord { x: x_offset, y: 0 }), fb);
        StyledString {
            string: format!("< {:.0}% >", state.config.sfx_volume * 100.),
            style,
        }
        .render(&(), ctx.add_offset(Coord { x: x_offset, y: 1 }), fb);
    }

    fn update(&mut self, state: &mut Self::State, ctx: Ctx, event: Event) -> Self::Output {
        let mut update_volume = |volume_delta: f32| {
            let volume = match self.menu.selected() {
                OptionsMenuEntry::MusicVolume => &mut state.config.music_volume,
                OptionsMenuEntry::SfxVolume => &mut state.config.sfx_volume,
                OptionsMenuEntry::Back => return,
            };
            *volume = (*volume + volume_delta).clamp(0., 1.);
            state
                .audio_state
                .set_music_volume(state.config.music_volume);
            state.save_config();
        };
        if let Some(input_policy) = event.input_policy() {
            match input_policy {
                InputPolicy::Left => update_volume(-0.05),
                InputPolicy::Right => update_volume(0.05),
                InputPolicy::Select => {
                    // prevent hitting enter on a menu option from closing the menu
                    if OptionsMenuEntry::Back != *self.menu.selected() {
                        return None;
                    }
                }
                _ => (),
            }
        }
        self.menu.update(&mut (), ctx, event).map(|_| ())
    }

    fn size(&self, _state: &Self::State, ctx: Ctx) -> Size {
        self.menu.size(&(), ctx) + Size::new(9, 0)
    }
}
fn options_menu() -> CF<()> {
    use menu::builder::*;
    use OptionsMenuEntry::*;
    let mut builder = menu_builder();
    let add_item = |builder: &mut MenuBuilder<_>, entry, name| {
        let identifier = MENU_FADE_SPEC.identifier(move |b| write!(b, "{}", name).unwrap());
        builder.add_item_mut(item(entry, identifier));
    };
    add_item(&mut builder, MusicVolume, "Music Volume:");
    add_item(&mut builder, SfxVolume, "SFX Volume:");
    builder.add_space_mut();
    add_item(&mut builder, Back, "Back");
    let menu = builder.build();
    boxed_cf(OptionsMenuComponent { menu })
        .catch_escape()
        .map(|_| ())
}

fn game_instance_component(running: witness::Running) -> CF<GameLoopState> {
    boxed_cf(GameInstanceComponent::new(running)).some()
}

pub enum GameExitReason {
    GameOver,
    Quit,
}

fn first_run_prologue() -> CF<()> {
    on_state_then(|state: &mut GameLoopData| {
        if state.config.first_run {
            state.config.first_run = false;
            state.save_config();
            text::prologue(MAIN_MENU_TEXT_WIDTH)
                .centre()
                .bound_width(42)
                .overlay(MenuBackgroundComponent, chargrid::core::TintIdentity, 10)
        } else {
            unit().some()
        }
    })
}

pub fn game_loop_component(initial_state: GameLoopState) -> CF<GameExitReason> {
    use GameLoopState::*;
    first_run_prologue().and_then(|()| {
        loop_(initial_state, |state| match state {
            Playing(witness) => match witness {
                Witness::Running(running) => game_instance_component(running).continue_(),
                Witness::Upgrade(upgrade) => {
                    try_upgrade_component(upgrade).map(Playing).continue_()
                }
                Witness::GetRangedWeapon(get_ranged_weapon) => todo!(),
                Witness::GetMeleeWeapon(get_melee_weapon) => todo!(),
                Witness::GameOver => break_(GameExitReason::GameOver),
            },
            Paused(running) => pause(running).map(|pause_output| match pause_output {
                PauseOutput::ContinueGame { running } => {
                    LoopControl::Continue(Playing(running.into_witness()))
                }
                PauseOutput::MainMenu => LoopControl::Continue(MainMenu),
                PauseOutput::Quit => LoopControl::Break(GameExitReason::Quit),
            }),
            MainMenu => main_menu_loop().map(|main_menu_output| match main_menu_output {
                MainMenuOutput::NewGame { new_running } => {
                    LoopControl::Continue(Playing(new_running.into_witness()))
                }
                MainMenuOutput::Quit => LoopControl::Break(GameExitReason::Quit),
            }),
        })
        .bound_size(Size::new_u16(80, 60))
    })
}
