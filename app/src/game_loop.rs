use crate::audio::{AppAudioPlayer, Audio, AudioState};
use crate::{
    colours,
    controls::{AppInput, Controls},
    examine,
    game_instance::{GameInstance, GameInstanceStorable},
    menu_background::MenuBackground,
    text, ui,
};
use chargrid::{
    border::BorderStyle, control_flow::boxed::*, input::*, menu, menu::Menu, pad_by::Padding,
    prelude::*, text::StyledString,
};
use general_storage_static::{format, StaticStorage};
use orbital_decay_game::{
    player,
    witness::{self, GameOver, GameOverType, Witness},
    ActionError, Config as GameConfig, ExternalEvent, Game, Music, MAP_SIZE,
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
pub type State = GameLoopData;

const CURSOR_COLOUR: Rgba32 = Rgba32::new(255, 255, 0, 64);
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
    pub controls_key: String,
}

impl AppStorage {
    const SAVE_GAME_STORAGE_FORMAT: format::Bincode = format::Bincode;
    const CONFIG_STORAGE_FORMAT: format::JsonPretty = format::JsonPretty;
    const CONTROLS_STORAGE_FORMAT: format::JsonPretty = format::JsonPretty;

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

    fn save_controls(&mut self, controls: &Controls) {
        let result =
            self.handle
                .store(&self.controls_key, &controls, Self::CONTROLS_STORAGE_FORMAT);
        if let Err(e) = result {
            use general_storage_static::{StoreError, StoreRawError};
            match e {
                StoreError::FormatError(e) => log::error!("Failed to format controls: {}", e),
                StoreError::Raw(e) => match e {
                    StoreRawError::IoError(e) => {
                        log::error!("Error while writing controls: {}", e)
                    }
                },
            }
        }
    }

    fn load_controls(&self) -> Option<Controls> {
        let result = self
            .handle
            .load::<_, Controls, _>(&self.controls_key, Self::CONTROLS_STORAGE_FORMAT);
        match result {
            Err(e) => {
                use general_storage_static::{LoadError, LoadRawError};
                match e {
                    LoadError::FormatError(e) => {
                        log::error!("Failed to parse controls file: {}", e)
                    }
                    LoadError::Raw(e) => match e {
                        LoadRawError::IoError(e) => {
                            log::error!("Error while reading controls: {}", e)
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

fn action_error_message(action_error: ActionError) -> StyledString {
    let style = Style::plain_text();
    let string = match action_error {
        ActionError::WalkIntoSolidCell => "You can't walk there!".to_string(),
        ActionError::CannotAffordUpgrade => "You can't afford that!".to_string(),
        ActionError::NoItemToGet => "There is no item here!".to_string(),
        ActionError::NoWeaponInSlot(slot) => format!("No weapon in slot {}!", slot.number()),
        ActionError::WeaponOutOfAmmo(name) => {
            format!("{} is out of ammo!", ui::weapon_name_text(name).string)
        }
    };
    StyledString { string, style }
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
    context_message: Option<StyledString>,
    examine_message: Option<StyledString>,
    cursor: Option<Coord>,
    duration: Duration,
}

impl GameLoopData {
    pub fn new(
        game_config: GameConfig,
        mut storage: AppStorage,
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
        let controls = if let Some(controls) = storage.load_controls() {
            controls
        } else {
            let controls = Controls::default();
            storage.save_controls(&controls);
            controls
        };
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
                context_message: None,
                examine_message: None,
                cursor: None,
                duration: Duration::from_millis(0),
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

    fn render(&self, cursor_colour: Rgba32, ctx: Ctx, fb: &mut FrameBuffer) {
        let instance = self.instance.as_ref().unwrap();
        instance.render(ctx, fb);
        if let Some(cursor) = self.cursor {
            if cursor.is_valid(MAP_SIZE + Size::new_u16(1, 1)) {
                let screen_cursor = cursor * 3;
                for offset in Size::new_u16(3, 3).coord_iter_row_major() {
                    fb.set_cell_relative_to_ctx(
                        ctx,
                        screen_cursor + offset,
                        10,
                        RenderCell::BLANK.with_background(cursor_colour),
                    );
                }
            }
        }
        self.render_text(ctx, fb);
    }

    fn render_text(&self, ctx: Ctx, fb: &mut FrameBuffer) {
        let instance = self.instance.as_ref().unwrap();
        if let Some(context_message) = self.context_message.as_ref() {
            context_message.render(&(), ctx.add_y(1), fb);
        }
        if let Some(top_text) = self.examine_message.as_ref() {
            top_text.clone().wrap_word().render(&(), ctx, fb);
        } else {
            instance.floor_text().render(&(), ctx, fb);
        }
    }

    fn examine_mouse(&mut self, event: Event) {
        match event {
            Event::Input(Input::Mouse(mouse_input)) => match mouse_input {
                MouseInput::MouseMove { button: _, coord } => {
                    self.cursor = Some(coord / 3);
                }
                _ => (),
            },
            _ => (),
        }
    }

    fn update_examine_text(&mut self) {
        self.examine_message = self
            .cursor
            .and_then(|coord| examine::examine(self.game().inner_ref(), coord));
    }

    fn update(&mut self, event: Event, running: witness::Running) -> GameLoopState {
        let instance = self.instance.as_mut().unwrap();
        let witness = match event {
            Event::Input(input) => {
                if let Some(app_input) = self.controls.get(input) {
                    let (witness, action_result) = match app_input {
                        AppInput::Direction(direction) => {
                            running.walk(&mut instance.game, direction, &self.game_config)
                        }
                        AppInput::Wait => running.wait(&mut instance.game, &self.game_config),
                        AppInput::Get => running.get(&instance.game),
                        AppInput::Slot(slot) => running.fire_weapon(&instance.game, slot),
                        AppInput::Examine => {
                            return GameLoopState::Examine(running);
                        }
                    };
                    if let Err(action_error) = action_result {
                        self.context_message = Some(action_error_message(action_error));
                    } else {
                        self.context_message = None;
                    }
                    witness
                } else {
                    running.into_witness()
                }
            }
            Event::Tick(since_previous) => {
                running.tick(&mut instance.game, since_previous, &self.game_config)
            }
            _ => Witness::Running(running),
        };
        self.examine_mouse(event);
        self.update_examine_text();
        self.handle_game_events();
        GameLoopState::Playing(witness)
    }

    fn game_over_tick(&mut self, event: Event, game_over: GameOver) -> GameOver {
        const NPC_TURN_PERIOD: Duration = Duration::from_millis(100);
        let instance = self.instance.as_mut().unwrap();
        match event {
            Event::Tick(since_previous) => {
                self.duration += since_previous;
                if self.duration > NPC_TURN_PERIOD {
                    self.duration -= NPC_TURN_PERIOD;
                    instance.game.npc_turn();
                }
                game_over.tick(&mut instance.game, since_previous, &self.game_config)
            }
            _ => game_over,
        }
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
                ExternalEvent::SoundEffect(sound_effect) => {
                    self.audio_state
                        .play_once(Audio::SoundEffect(sound_effect), self.config.sfx_volume);
                }
                _ => (),
            }
        }
    }

    fn game(&self) -> &witness::Game {
        &self.instance.as_ref().unwrap().game
    }

    fn game_mut_config(&mut self) -> (&mut witness::Game, &GameConfig) {
        (&mut self.instance.as_mut().unwrap().game, &self.game_config)
    }

    fn game_inner(&self) -> &Game {
        self.game().inner_ref()
    }

    fn player_has_third_weapon_slot(&self) -> bool {
        self.game_inner().player_has_third_weapon_slot()
    }

    fn player_has_weapon_in_slot(&self, slot: player::RangedWeaponSlot) -> bool {
        self.game_inner().player_has_weapon_in_slot(slot)
    }
}

struct GameInstanceComponent(Option<witness::Running>);

impl GameInstanceComponent {
    fn new(running: witness::Running) -> Self {
        Self(Some(running))
    }
}

pub enum GameLoopState {
    Examine(witness::Running),
    Paused(witness::Running),
    Playing(Witness),
    MainMenu,
}

impl Component for GameInstanceComponent {
    type Output = GameLoopState;
    type State = GameLoopData;

    fn render(&self, state: &Self::State, ctx: Ctx, fb: &mut FrameBuffer) {
        state.render(CURSOR_COLOUR, ctx, fb);
    }

    fn update(&mut self, state: &mut Self::State, _ctx: Ctx, event: Event) -> Self::Output {
        let running = self.0.take().unwrap();
        if event.is_escape_or_start() {
            GameLoopState::Paused(running)
        } else {
            state.update(event, running)
        }
    }

    fn size(&self, _state: &Self::State, ctx: Ctx) -> Size {
        ctx.bounding_box.size()
    }
}

struct GameExamineWithMouseComponent;

impl Component for GameExamineWithMouseComponent {
    type Output = ();
    type State = GameLoopData;

    fn render(&self, state: &Self::State, ctx: Ctx, fb: &mut FrameBuffer) {
        state.render(CURSOR_COLOUR, ctx, fb);
    }

    fn update(&mut self, state: &mut Self::State, _ctx: Ctx, event: Event) -> Self::Output {
        state.examine_mouse(event);
    }

    fn size(&self, _state: &Self::State, ctx: Ctx) -> Size {
        ctx.bounding_box.size()
    }
}

struct GameExamineComponent;

impl Component for GameExamineComponent {
    type Output = Option<()>;
    type State = GameLoopData;

    fn render(&self, state: &Self::State, ctx: Ctx, fb: &mut FrameBuffer) {
        state.render(CURSOR_COLOUR.with_a(128), ctx, fb);
    }

    fn update(&mut self, state: &mut Self::State, _ctx: Ctx, event: Event) -> Self::Output {
        if let Some(input) = event.input() {
            state.controls.get_direction(input).map(|direction| {
                let cursor = state
                    .cursor
                    .unwrap_or_else(|| state.game().inner_ref().player_coord());
                state.cursor = Some(cursor + direction.coord());
            });
            if let Some(AppInput::Examine) = state.controls.get(input) {
                return Some(());
            }
        }
        state.examine_mouse(event);
        state.update_examine_text();
        None
    }

    fn size(&self, _state: &Self::State, ctx: Ctx) -> Size {
        ctx.bounding_box.size()
    }
}

fn game_examine_component() -> CF<()> {
    on_state_then(|state: &mut State| {
        state.context_message = Some(StyledString {
            string: "Examining (escape/start to return to game)".to_string(),
            style: Style::plain_text().with_foreground(Rgba32::new_grey(100)),
        });
        let cursor = state
            .cursor
            .unwrap_or_else(|| state.game().inner_ref().player_coord());
        state.cursor = Some(cursor);
        boxed_cf(GameExamineComponent)
            .catch_escape_or_start()
            .map_val(|| ())
            .then_side_effect(|state: &mut State| {
                state.context_message = None;
                state.cursor = None;
            })
    })
}

struct GameOverComponent(Option<witness::GameOver>);

struct TintAdrift;
impl Tint for TintAdrift {
    fn tint(&self, rgba32: Rgba32) -> Rgba32 {
        let mean = rgba32
            .to_rgb24()
            .weighted_mean_u16(rgb_int::WeightsU16::new(1, 1, 1));
        Rgba32::new_rgb(0, 0, mean).saturating_scalar_mul_div(3, 2)
    }
}

struct TintDead;
impl Tint for TintDead {
    fn tint(&self, rgba32: Rgba32) -> Rgba32 {
        let mean = rgba32
            .to_rgb24()
            .weighted_mean_u16(rgb_int::WeightsU16::new(1, 1, 1));
        Rgba32::new_rgb(mean, 0, 0).saturating_scalar_mul_div(3, 2)
    }
}

impl Component for GameOverComponent {
    type Output = ();
    type State = GameLoopData;

    fn render(&self, state: &Self::State, ctx: Ctx, fb: &mut FrameBuffer) {
        let instance = state.instance.as_ref().unwrap();
        match self.0.as_ref().unwrap().typ() {
            GameOverType::Adrift => instance.render_omniscient(ctx_tint!(ctx, TintAdrift), fb),
            GameOverType::Dead => instance.render_omniscient(ctx_tint!(ctx, TintDead), fb),
        }
        instance.render_message_log(ctx, fb);
        instance.render_hud(ctx, fb);
        state.render_text(ctx, fb);
    }

    fn update(&mut self, state: &mut Self::State, _ctx: Ctx, event: Event) -> Self::Output {
        let game_over = self.0.take().unwrap();
        self.0 = Some(state.game_over_tick(event, game_over));
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
        .overlay_tint(
            render_state(|state: &State, ctx, fb| state.render(CURSOR_COLOUR, ctx, fb)),
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
    on_state_then(|state: &mut State| {
        let instance = state.instance.as_ref().unwrap();
        let upgrades = instance.game.inner_ref().available_upgrades();
        use menu::builder::*;
        let mut builder = menu_builder().vi_keys();
        for upgrade in upgrades {
            let name = upgrade_identifier(upgrade);
            let identifier = MENU_FADE_SPEC.identifier(move |b| write!(b, "{}", name).unwrap());
            builder = builder.add_item(item(upgrade, identifier));
        }
        let menu = builder.build();
        UpgradeMenuDecorated { menu }
    })
}

fn yes_no_menu() -> CF<bool> {
    use menu::builder::*;
    menu_builder()
        .vi_keys()
        .add_item(
            item(
                true,
                MENU_FADE_SPEC.identifier(move |b| write!(b, "(y) Yes").unwrap()),
            )
            .add_hotkey_char('y')
            .add_hotkey_char('Y'),
        )
        .add_item(
            item(
                false,
                MENU_FADE_SPEC.identifier(move |b| write!(b, "(n) No").unwrap()),
            )
            .add_hotkey_char('n')
            .add_hotkey_char('N'),
        )
        .build_boxed_cf()
}

fn yes_no(message: String) -> CF<bool> {
    menu_style(
        yes_no_menu().with_title(
            boxed_cf(
                StyledString {
                    string: message,
                    style: Style::plain_text(),
                }
                .wrap_word(),
            )
            .ignore_state()
            .bound_width(40),
            1,
        ),
    )
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
        .menu_harness()
        .and_then(|result| {
            on_state_then(move |state: &mut State| match result {
                Err(Close) => val_once(upgrade_witness.cancel()),
                Ok(upgrade) => {
                    let instance = state.instance.as_mut().unwrap();
                    if upgrade.level.cost() > instance.game.player().credit {
                        popup("You can't afford that!".to_string())
                            .map_val(|| upgrade_witness.cancel())
                    } else {
                        let (witness, result) =
                            upgrade_witness.commit(&mut instance.game, upgrade, &state.game_config);
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
    on_state_then(move |state: &mut State| {
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
    on_state_then(move |state: &mut State| {
        let num_weapon_slots = if state.player_has_third_weapon_slot() {
            3
        } else {
            2
        };
        state.context_message = Some(StyledString {
            string: format!(
                "Choose a weapon slot: (press 1-{} or escape/start to cancel)",
                num_weapon_slots
            ),
            style: Style::plain_text()
                .with_bold(true)
                .with_foreground(Rgba32::hex_rgb(0xFF0000)),
        });
        on_input_state(move |input, state: &mut State| {
            use player::RangedWeaponSlot::*;
            let slot = state.controls.get_slot(input);
            if slot == Some(Slot3) && num_weapon_slots < 3 {
                None
            } else {
                slot
            }
        })
        .catch_escape_or_start()
        .overlay(
            render_state(|state: &State, ctx, fb| state.render(CURSOR_COLOUR, ctx, fb)),
            10,
        )
        .and_then(|slot_or_err| {
            on_state_then(move |state: &mut State| {
                state.context_message = None;
                match slot_or_err {
                    Err(_escape_or_start) => val_once(witness.cancel()),
                    Ok(slot) => {
                        if state.player_has_weapon_in_slot(slot) {
                            yes_no(format!("Replace ranged weapon in slot {}?", slot.number()))
                                .and_then(move |yes| {
                                    on_state(move |state: &mut State| {
                                        if yes {
                                            let (game, config) = state.game_mut_config();
                                            witness.commit(game, slot, config)
                                        } else {
                                            witness.cancel()
                                        }
                                    })
                                })
                        } else {
                            let (game, config) = state.game_mut_config();
                            val_once(witness.commit(game, slot, config))
                        }
                    }
                }
            })
        })
    })
}

fn try_get_melee_weapon(witness: witness::GetMeleeWeapon) -> CF<Witness> {
    yes_no("Replace current melee weapon?".to_string()).and_then(move |yes| {
        on_state(move |state: &mut State| {
            if yes {
                let (game, config) = state.game_mut_config();
                witness.commit(game, config)
            } else {
                witness.cancel()
            }
        })
    })
}

fn fire_weapon(witness: witness::FireWeapon) -> CF<Witness> {
    on_state_then(move |state: &mut State| {
        state.context_message = Some(StyledString {
            string: format!(
                "Fire weapon {} in which direction? (escape/start to cancel)",
                witness.slot().number()
            ),
            style: Style::plain_text()
                .with_bold(true)
                .with_foreground(Rgba32::hex_rgb(0xFF0000)),
        });
        on_input_state(move |input, state: &mut State| state.controls.get_direction(input))
            .catch_escape_or_start()
            .overlay(GameExamineWithMouseComponent, 10)
            .and_then(|direction_or_err| {
                on_state(move |state: &mut State| {
                    state.context_message = None;
                    match direction_or_err {
                        Err(_escape_or_start) => witness.cancel(),
                        Ok(direction) => {
                            let (game, config) = state.game_mut_config();
                            witness.commit(game, direction, config)
                        }
                    }
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
        .overlay(decoration, 10)
}

fn main_menu() -> CF<MainMenuEntry> {
    on_state_then(|state: &mut State| {
        use menu::builder::*;
        use MainMenuEntry::*;
        let mut builder = menu_builder().vi_keys();
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
            NewGame => on_state(|state: &mut State| MainMenuOutput::NewGame {
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
        .overlay(MenuBackgroundComponent, 10)
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
    on_state_then(|state: &mut State| {
        use menu::builder::*;
        use PauseMenuEntry::*;
        let mut builder = menu_builder().vi_keys();
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
        .menu_harness()
        .repeat(
            running,
            move |running, entry_or_escape| match entry_or_escape {
                Ok(entry) => match entry {
                    Resume => break_(PauseOutput::ContinueGame { running }),
                    SaveQuit => on_state(|state: &mut State| {
                        state.save_instance(running);
                        PauseOutput::Quit
                    })
                    .break_(),
                    Save => on_state(|state: &mut State| PauseOutput::ContinueGame {
                        running: state.save_instance(running),
                    })
                    .break_(),
                    NewGame => on_state(|state: &mut State| PauseOutput::ContinueGame {
                        running: state.new_game(),
                    })
                    .break_(),
                    Options => options_menu().continue_with(running),
                    Help => text::help(text_width).continue_with(running),
                    Prologue => text::prologue(text_width).continue_with(running),
                    Epilogue => text::epilogue(text_width).continue_with(running),
                    Clear => on_state(|state: &mut State| {
                        state.clear_saved_game();
                        PauseOutput::MainMenu
                    })
                    .break_(),
                },
                Err(_escape_or_start) => break_(PauseOutput::ContinueGame { running }),
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
    on_state_then(move |state: &mut State| {
        // turn down the music in the pause menu
        state
            .audio_state
            .set_music_volume_multiplier(PAUSE_MUSIC_VOLUME_MULTIPLIER);
        menu_style(pause_menu_loop(running))
    })
    .then_side_effect(|state: &mut State| {
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
    let mut builder = menu_builder().vi_keys();
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
        .catch_escape_or_start()
        .map(|_| ())
}

fn game_instance_component(running: witness::Running) -> CF<GameLoopState> {
    boxed_cf(GameInstanceComponent::new(running))
        .some()
        .no_peek()
}

fn first_run_prologue() -> CF<()> {
    on_state_then(|state: &mut State| {
        if state.config.first_run {
            state.config.first_run = false;
            state.save_config();
            text::prologue(MAIN_MENU_TEXT_WIDTH)
                .centre()
                .bound_width(42)
                .overlay(MenuBackgroundComponent, 10)
        } else {
            unit().some()
        }
    })
}

fn game_over(game_over_witness: GameOver) -> CF<()> {
    on_state_then(move |state: &mut State| {
        state.examine_message = Some(StyledString {
            string: "You drift in space forever! Press any key...".to_string(),
            style: Style::plain_text().with_foreground(Rgba32::hex_rgb(0xFF0000)),
        });
        GameOverComponent(Some(game_over_witness))
    })
    .press_any_key()
    .then_side_effect(|state: &mut State| {
        state
            .audio_state
            .loop_music(Audio::Menu, state.config.music_volume);
    })
}

fn unlock_map(witness: witness::UnlockMap) -> CF<Witness> {
    yes_no("Spend $2 credit to unlock map?".to_string()).and_then(move |yes| {
        on_state(move |state: &mut State| {
            if yes {
                let (game, config) = state.game_mut_config();
                witness.commit(game, config)
            } else {
                witness.cancel()
            }
        })
    })
}

pub fn game_loop_component(initial_state: GameLoopState) -> CF<()> {
    use GameLoopState::*;
    first_run_prologue().then(|| {
        loop_(initial_state, |state| match state {
            Playing(witness) => match witness {
                Witness::Running(running) => game_instance_component(running).continue_(),
                Witness::Upgrade(upgrade) => {
                    try_upgrade_component(upgrade).map(Playing).continue_()
                }
                Witness::GetRangedWeapon(get_ranged_weapon) => {
                    try_get_ranged_weapon(get_ranged_weapon)
                        .map(Playing)
                        .continue_()
                }
                Witness::GetMeleeWeapon(get_melee_weapon) => try_get_melee_weapon(get_melee_weapon)
                    .map(Playing)
                    .continue_(),
                Witness::FireWeapon(fire_weapon_witness) => {
                    fire_weapon(fire_weapon_witness).map(Playing).continue_()
                }
                Witness::GameOver(game_over_witness) => game_over(game_over_witness)
                    .map_val(|| MainMenu)
                    .continue_(),
                Witness::UnlockMap(unlock_map_witness) => {
                    unlock_map(unlock_map_witness).map(Playing).continue_()
                }
            },
            Examine(running) => game_examine_component()
                .map_val(|| Playing(running.into_witness()))
                .continue_(),
            Paused(running) => pause(running).map(|pause_output| match pause_output {
                PauseOutput::ContinueGame { running } => {
                    LoopControl::Continue(Playing(running.into_witness()))
                }
                PauseOutput::MainMenu => LoopControl::Continue(MainMenu),
                PauseOutput::Quit => LoopControl::Break(()),
            }),
            MainMenu => main_menu_loop().map(|main_menu_output| match main_menu_output {
                MainMenuOutput::NewGame { new_running } => {
                    LoopControl::Continue(Playing(new_running.into_witness()))
                }
                MainMenuOutput::Quit => LoopControl::Break(()),
            }),
        })
        .bound_size(Size::new_u16(80, 60))
    })
}
