use crate::audio::{AppAudioPlayer, Audio};
use crate::colours;
use crate::controls::Controls;
use crate::depth;
use crate::frontend::Frontend;
use crate::game::{
    AimEventRoutine, ChooseWeaponSlotEventRoutine, ExamineEventRoutine, Fire, GameData,
    GameEventRoutine, GameOverEventRoutine, GameReturn, GameStatus, InjectedInput, ScreenCoord,
};
pub use crate::game::{GameConfig, Omniscient, RngSeed};
use crate::menu_background::MenuBackgroundData;
use crate::render::{GameToRender, GameView, Mode};
use chargrid::input::*;
use chargrid::*;
use common_event::*;
use decorator::*;
use event_routine::*;
use general_storage_static::StaticStorage;
use maplit::hashmap;
use menu::{
    fade_spec, FadeMenuInstanceView, MenuEntryStringFn, MenuEntryToRender, MenuInstanceChoose,
};
use orbital_decay_game::player::{self, RangedWeaponSlot};
use render::{ColModifyDefaultForeground, ColModifyMap, Coord, Rgb24, Style};
use std::time::Duration;

#[derive(Clone, Copy)]
enum MainMenuType {
    Init,
    Pause,
}

#[derive(Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
enum MainMenuEntry {
    NewGame,
    Resume,
    Quit,
    Save,
    SaveQuit,
    Clear,
    Options,
    Story,
    Help,
    EndText,
}

impl MainMenuEntry {
    fn init(frontend: Frontend) -> menu::MenuInstance<Self> {
        use MainMenuEntry::*;
        let (items, hotkeys) = match frontend {
            Frontend::Graphical | Frontend::AnsiTerminal => (
                vec![NewGame, Options, Help, Story, Quit],
                hashmap!['n' => NewGame, 'o' => Options, 'h' => Help, 'b' => Story, 'q' => Quit],
            ),
            Frontend::Web => (
                vec![NewGame, Options, Help, Story],
                hashmap!['n' => NewGame, 'o' => Options, 'h' => Help, 'b' => Story],
            ),
        };
        menu::MenuInstanceBuilder {
            items,
            selected_index: 0,
            hotkeys: Some(hotkeys),
        }
        .build()
        .unwrap()
    }
    fn won(frontend: Frontend) -> menu::MenuInstance<Self> {
        use MainMenuEntry::*;
        let (items, hotkeys) = match frontend {
            Frontend::Graphical | Frontend::AnsiTerminal => (
                vec![NewGame, Options, Help, Story, EndText, Quit],
                hashmap!['n' => NewGame, 'o' => Options, 'h' => Help, 'b' => Story, 'e' => EndText, 'q' => Quit],
            ),
            Frontend::Web => (
                vec![NewGame, Options, Help, Story, EndText],
                hashmap!['n' => NewGame, 'o' => Options, 'h' => Help, 'b' => Story, 'e' => EndText],
            ),
        };
        menu::MenuInstanceBuilder {
            items,
            selected_index: 0,
            hotkeys: Some(hotkeys),
        }
        .build()
        .unwrap()
    }
    fn pause(frontend: Frontend) -> menu::MenuInstance<Self> {
        use MainMenuEntry::*;
        let (items, hotkeys) = match frontend {
            Frontend::Graphical | Frontend::AnsiTerminal => (
                vec![Resume, SaveQuit, NewGame, Options, Help, Story, Clear],
                hashmap!['r' => Resume, 'q' => SaveQuit, 'o' => Options, 'h' => Help, 'b'=> Story, 'n' => NewGame, 'c' => Clear],
            ),
            Frontend::Web => (
                vec![Resume, Save, NewGame, Options, Help, Story, Clear],
                hashmap!['r' => Resume, 's' => Save, 'o' => Options, 'h' => Help, 'b' => Story, 'n' => NewGame, 'c' => Clear],
            ),
        };
        menu::MenuInstanceBuilder {
            items,
            selected_index: 0,
            hotkeys: Some(hotkeys),
        }
        .build()
        .unwrap()
    }
}

struct AppData {
    frontend: Frontend,
    game: GameData,
    main_menu: menu::MenuInstanceChooseOrEscape<MainMenuEntry>,
    main_menu_type: MainMenuType,
    upgrade_menu: Option<menu::MenuInstanceChooseOrEscape<player::Upgrade>>,
    options_menu: menu::MenuInstanceChooseOrEscape<OrBack<OptionsMenuEntry>>,
    confirm_menu: menu::MenuInstanceChooseOrEscape<bool>,
    last_mouse_coord: Coord,
    env: Box<dyn Env>,
    menu_background_data: MenuBackgroundData,
    prime_font_countdown: Duration,
}

struct AppView {
    game: GameView,
    main_menu: FadeMenuInstanceView,
    options_menu: FadeMenuInstanceView,
    upgrade_menu: FadeMenuInstanceView,
    confirm_menu: FadeMenuInstanceView,
}

impl AppData {
    fn new(
        game_config: GameConfig,
        frontend: Frontend,
        controls: Controls,
        storage: StaticStorage,
        save_key: String,
        audio_player: AppAudioPlayer,
        rng_seed: RngSeed,
        fullscreen: Option<Fullscreen>,
        env: Box<dyn Env>,
    ) -> Self {
        let mut game_data = GameData::new(
            game_config,
            controls,
            storage,
            save_key,
            audio_player,
            rng_seed,
            frontend,
        );
        if env.fullscreen_supported() {
            let mut config = game_data.config();
            if fullscreen.is_some() {
                config.fullscreen = true;
            }
            env.set_fullscreen_init(config.fullscreen);
            game_data.set_config(config);
        }
        let menu_background_data = MenuBackgroundData::new();
        Self {
            confirm_menu: menu::MenuInstanceBuilder {
                items: vec![true, false],
                selected_index: 0,
                hotkeys: Some(hashmap!['y' => true, 'n' => false]),
            }
            .build()
            .unwrap()
            .into_choose_or_escape(),
            options_menu: OptionsMenuEntry::instance(&env),
            frontend,
            game: game_data,
            main_menu: MainMenuEntry::init(frontend).into_choose_or_escape(),
            main_menu_type: MainMenuType::Init,
            upgrade_menu: None,
            last_mouse_coord: Coord::new(0, 0),
            env,
            menu_background_data,
            prime_font_countdown: Duration::from_millis(100),
        }
    }
}

impl AppView {
    fn new() -> Self {
        use fade_spec::*;
        let spec = Spec {
            normal: Style {
                to: To {
                    foreground: colours::STRIPE,
                    background: colours::SPACE_BACKGROUND.saturating_scalar_mul_div(2, 3),
                    bold: false,
                    underline: false,
                },
                from: From::current(),
                durations: Durations {
                    foreground: Duration::from_millis(128),
                    background: Duration::from_millis(128),
                },
            },
            selected: Style {
                to: To {
                    foreground: colours::WALL_TOP,
                    background: colours::STRIPE,
                    bold: true,
                    underline: false,
                },
                from: From::current(),
                durations: Durations {
                    foreground: Duration::from_millis(128),
                    background: Duration::from_millis(128),
                },
            },
        };
        Self {
            game: GameView::new(),
            main_menu: FadeMenuInstanceView::new(spec.clone()),
            options_menu: FadeMenuInstanceView::new(spec.clone()),
            upgrade_menu: FadeMenuInstanceView::new(spec.clone()),
            confirm_menu: FadeMenuInstanceView::new(spec.clone()),
        }
    }
}

impl Default for AppView {
    fn default() -> Self {
        Self::new()
    }
}

struct SelectGame;
impl DataSelector for SelectGame {
    type DataInput = AppData;
    type DataOutput = GameData;
    fn data<'a>(&self, input: &'a Self::DataInput) -> &'a Self::DataOutput {
        &input.game
    }
    fn data_mut<'a>(&self, input: &'a mut Self::DataInput) -> &'a mut Self::DataOutput {
        &mut input.game
    }
}
impl ViewSelector for SelectGame {
    type ViewInput = AppView;
    type ViewOutput = GameView;
    fn view<'a>(&self, input: &'a Self::ViewInput) -> &'a Self::ViewOutput {
        &input.game
    }
    fn view_mut<'a>(&self, input: &'a mut Self::ViewInput) -> &'a mut Self::ViewOutput {
        &mut input.game
    }
}
impl Selector for SelectGame {}

struct SelectMainMenu;
impl ViewSelector for SelectMainMenu {
    type ViewInput = AppView;
    type ViewOutput = FadeMenuInstanceView;
    fn view<'a>(&self, input: &'a Self::ViewInput) -> &'a Self::ViewOutput {
        &input.main_menu
    }
    fn view_mut<'a>(&self, input: &'a mut Self::ViewInput) -> &'a mut Self::ViewOutput {
        &mut input.main_menu
    }
}
impl DataSelector for SelectMainMenu {
    type DataInput = AppData;
    type DataOutput = menu::MenuInstanceChooseOrEscape<MainMenuEntry>;
    fn data<'a>(&self, input: &'a Self::DataInput) -> &'a Self::DataOutput {
        &input.main_menu
    }
    fn data_mut<'a>(&self, input: &'a mut Self::DataInput) -> &'a mut Self::DataOutput {
        &mut input.main_menu
    }
}
impl Selector for SelectMainMenu {}

struct DecorateMainMenu;

struct InitMenu<'e, 'v, E: EventRoutine>(EventRoutineView<'e, 'v, E>);
impl<'a, 'e, 'v, E> View<&'a AppData> for InitMenu<'e, 'v, E>
where
    E: EventRoutine<View = AppView, Data = AppData>,
{
    fn view<F: Frame, C: ColModify>(
        &mut self,
        app_data: &'a AppData,
        context: ViewContext<C>,
        frame: &mut F,
    ) {
        text::StringViewSingleLine::new(
            Style::new()
                .with_foreground(colours::WALL_FRONT)
                .with_bold(true),
        )
        .view("Orbital Decay", context, frame);
        self.0
            .view(app_data, context.add_offset(Coord::new(0, 4)), frame);
    }
}

struct SelectUpgradeMenu;
impl ViewSelector for SelectUpgradeMenu {
    type ViewInput = AppView;
    type ViewOutput = FadeMenuInstanceView;
    fn view<'a>(&self, input: &'a Self::ViewInput) -> &'a Self::ViewOutput {
        &input.upgrade_menu
    }
    fn view_mut<'a>(&self, input: &'a mut Self::ViewInput) -> &'a mut Self::ViewOutput {
        &mut input.upgrade_menu
    }
}
impl DataSelector for SelectUpgradeMenu {
    type DataInput = AppData;
    type DataOutput = menu::MenuInstanceChooseOrEscape<player::Upgrade>;
    fn data<'a>(&self, input: &'a Self::DataInput) -> &'a Self::DataOutput {
        input.upgrade_menu.as_ref().unwrap()
    }
    fn data_mut<'a>(&self, input: &'a mut Self::DataInput) -> &'a mut Self::DataOutput {
        input.upgrade_menu.as_mut().unwrap()
    }
}
impl Selector for SelectUpgradeMenu {}

struct SelectConfirmMenu;
impl ViewSelector for SelectConfirmMenu {
    type ViewInput = AppView;
    type ViewOutput = FadeMenuInstanceView;
    fn view<'a>(&self, input: &'a Self::ViewInput) -> &'a Self::ViewOutput {
        &input.confirm_menu
    }
    fn view_mut<'a>(&self, input: &'a mut Self::ViewInput) -> &'a mut Self::ViewOutput {
        &mut input.confirm_menu
    }
}
impl DataSelector for SelectConfirmMenu {
    type DataInput = AppData;
    type DataOutput = menu::MenuInstanceChooseOrEscape<bool>;
    fn data<'a>(&self, input: &'a Self::DataInput) -> &'a Self::DataOutput {
        &input.confirm_menu
    }
    fn data_mut<'a>(&self, input: &'a mut Self::DataInput) -> &'a mut Self::DataOutput {
        &mut input.confirm_menu
    }
}
impl Selector for SelectConfirmMenu {}

struct TextOverlay {
    height: u32,
    text: Vec<text::RichTextPartOwned>,
}
impl TextOverlay {
    fn new(height: u32, text: Vec<text::RichTextPartOwned>) -> Self {
        Self { height, text }
    }
}
impl EventRoutine for TextOverlay {
    type Return = ();
    type Data = AppData;
    type View = AppView;
    type Event = CommonEvent;
    fn handle<EP>(
        self,
        _data: &mut Self::Data,
        _view: &Self::View,
        event_or_peek: EP,
    ) -> Handled<Self::Return, Self>
    where
        EP: EventOrPeek<Event = Self::Event>,
    {
        event_or_peek_with_handled(event_or_peek, self, |s, event| match event {
            CommonEvent::Input(input) => match input {
                Input::Keyboard(_) | Input::Gamepad(_) => Handled::Return(()),
                Input::Mouse(_) => Handled::Continue(s),
            },
            CommonEvent::Frame(_) => Handled::Continue(s),
        })
    }
    fn view<F, C>(
        &self,
        data: &Self::Data,
        view: &mut Self::View,
        context: ViewContext<C>,
        frame: &mut F,
    ) where
        F: Frame,
        C: ColModify,
    {
        if let Some(instance) = data.game.instance() {
            AlignView {
                alignment: Alignment::centre(),
                view: FillBackgroundView {
                    rgb24: colours::SPACE_BACKGROUND.saturating_scalar_mul_div(2, 3),
                    view: BorderView {
                        style: &BorderStyle {
                            padding: BorderPadding::all(1),
                            ..Default::default()
                        },
                        view: BoundView {
                            size: Size::new(40, self.height),
                            view: text::RichTextView::new(text::wrap::Word::new()),
                        },
                    },
                },
            }
            .view(
                self.text.iter().map(|t| t.as_rich_text_part()),
                context.add_depth(depth::GAME_MAX + 1),
                frame,
            );
            view.game.view(
                GameToRender {
                    game: instance.game(),
                    status: GameStatus::Playing,
                    mouse_coord: None,
                    mode: Mode::Normal,
                    action_error: None,
                },
                context.compose_col_modify(
                    ColModifyDefaultForeground(Rgb24::new_grey(255)).compose(ColModifyMap(
                        |col: Rgb24| col.saturating_scalar_mul_div(1, 3),
                    )),
                ),
                frame,
            );
        } else {
            data.menu_background_data.render(context, frame);
            BoundView {
                size: Size::new(43, 60),
                view: AlignView {
                    alignment: Alignment::centre(),
                    view: BoundView {
                        size: Size::new(37, self.height),
                        view: text::RichTextView::new(text::wrap::Word::new()),
                    },
                },
            }
            .view(
                self.text.iter().map(|t| t.as_rich_text_part()),
                context,
                frame,
            );
        }
    }
}

impl Decorate for DecorateMainMenu {
    type View = AppView;
    type Data = AppData;
    fn view<E, F, C>(
        &self,
        data: &Self::Data,
        mut event_routine_view: EventRoutineView<E>,
        context: ViewContext<C>,
        frame: &mut F,
    ) where
        E: EventRoutine<Data = Self::Data, View = Self::View>,
        F: Frame,
        C: ColModify,
    {
        if let Some(instance) = data.game.instance() {
            AlignView {
                alignment: Alignment::centre(),
                view: FillBackgroundView {
                    rgb24: colours::SPACE_BACKGROUND.saturating_scalar_mul_div(2, 3),
                    view: BorderView {
                        style: &BorderStyle::new(),
                        view: &mut event_routine_view,
                    },
                },
            }
            .view(data, context.add_depth(depth::GAME_MAX + 1), frame);
            event_routine_view.view.game.view(
                GameToRender {
                    game: instance.game(),
                    status: GameStatus::Playing,
                    mouse_coord: None,
                    mode: Mode::Normal,
                    action_error: None,
                },
                context.compose_col_modify(
                    ColModifyDefaultForeground(Rgb24::new_grey(255)).compose(ColModifyMap(
                        |col: Rgb24| col.saturating_scalar_mul_div(1, 3),
                    )),
                ),
                frame,
            );
        } else {
            data.menu_background_data.render(context, frame);
            InitMenu(event_routine_view).view(
                &data,
                context.add_offset(Coord { x: 14, y: 24 }).add_depth(100),
                frame,
            );
            let mut credit_view = text::StringViewSingleLine::new(
                Style::new()
                    .with_foreground(colours::WALL_FRONT)
                    .with_bold(false),
            );
            let credit_y = 57;
            credit_view.view(
                "Programming and art by Stephen Sherratt",
                context.add_offset(Coord { x: 1, y: credit_y }),
                frame,
            );
            credit_view.view(
                "Music and sound effects by Lily Chen",
                context.add_offset(Coord {
                    x: 1,
                    y: credit_y + 1,
                }),
                frame,
            );
        }
    }
}

struct DecorateUpgradeMenu;

struct UpgradeMenuWindow<'a, 'b, 'c, E: EventRoutine>(&'a mut EventRoutineView<'b, 'c, E>);

impl<'a, 'b, 'c, E: EventRoutine<Data = AppData>> View<&'a AppData>
    for UpgradeMenuWindow<'a, 'b, 'c, E>
{
    fn view<F: Frame, C: ColModify>(
        &mut self,
        data: &'a AppData,
        context: ViewContext<C>,
        frame: &mut F,
    ) {
        let menu_instance = data.upgrade_menu.as_ref().unwrap().menu_instance();
        let balance = data.game.instance().unwrap().game().player().credit;
        text::StringViewSingleLine::new(Style::new().with_foreground(Rgb24::new_grey(255))).view(
            "Buy an Upgrade (escape cancels)",
            context,
            frame,
        );
        text::StringViewSingleLine::new(Style::new().with_foreground(Rgb24::new_grey(255))).view(
            format!("Your balance: ${}", balance),
            context.add_offset(Coord { x: 0, y: 2 }),
            frame,
        );
        self.0
            .view(data, context.add_offset(Coord { x: 0, y: 4 }), frame);
        let menu_length = menu_instance.len();
        use player::{Upgrade, UpgradeLevel::*, UpgradeType::*};
        let description = match menu_instance.selected() {
            Upgrade {
                typ: Toughness,
                level: Level1,
            } => "Toughness 1: Strong Back\nGain a third ranged weapon slot.",
            Upgrade {
                typ: Toughness,
                level: Level2,
            } => "Toughness 2: Hardy\nDouble your maximum health.",
            Upgrade {
                typ: Accuracy,
                level: Level1,
            } => "Accuracy 1: Careful\nReduce hull pen chance by half.",
            Upgrade {
                typ: Accuracy,
                level: Level2,
            } => "Accuracy 2: Kill Shot\nDeal double damage to enemies.",
            Upgrade {
                typ: Endurance,
                level: Level1,
            } => "Endurance 1: Sure-Footed\nVacuum pulls you one space each turn instead of two.",
            Upgrade {
                typ: Endurance,
                level: Level2,
            } => "Endurance 2: Big Lungs\nDouble your maximum oxygen.",
        };
        text::TextView::new(
            Style::new().with_foreground(Rgb24::new_grey(255)),
            text::wrap::Word::new(),
        )
        .view(
            vec![description],
            context.add_offset(Coord {
                x: 0,
                y: 5 + menu_length as i32,
            }),
            frame,
        );
    }
}

impl Decorate for DecorateUpgradeMenu {
    type View = AppView;
    type Data = AppData;
    fn view<E, F, C>(
        &self,
        data: &Self::Data,
        mut event_routine_view: EventRoutineView<E>,
        context: ViewContext<C>,
        frame: &mut F,
    ) where
        E: EventRoutine<Data = Self::Data, View = Self::View>,
        F: Frame,
        C: ColModify,
    {
        if let Some(instance) = data.game.instance() {
            AlignView {
                alignment: Alignment::centre(),
                view: FillBackgroundView {
                    rgb24: colours::SPACE_BACKGROUND.saturating_scalar_mul_div(2, 3),
                    view: BorderView {
                        style: &BorderStyle::new(),
                        view: BoundView {
                            size: Size::new(33, 12),
                            view: UpgradeMenuWindow(&mut event_routine_view),
                        },
                    },
                },
            }
            .view(data, context.add_depth(depth::GAME_MAX + 1), frame);
            event_routine_view.view.game.view(
                GameToRender {
                    game: instance.game(),
                    status: GameStatus::Playing,
                    mouse_coord: None,
                    mode: Mode::Normal,
                    action_error: None,
                },
                context.compose_col_modify(
                    ColModifyDefaultForeground(Rgb24::new_grey(255)).compose(ColModifyMap(
                        |col: Rgb24| col.saturating_scalar_mul_div(1, 3),
                    )),
                ),
                frame,
            );
        }
    }
}

struct ConfirmMenuWindow<'a, 'b, 'c, E: EventRoutine> {
    view: &'a mut EventRoutineView<'b, 'c, E>,
    text: &'a str,
}

impl<'a, 'b, 'c, E: EventRoutine<Data = AppData>> View<&'a AppData>
    for ConfirmMenuWindow<'a, 'b, 'c, E>
{
    fn view<F: Frame, C: ColModify>(
        &mut self,
        data: &'a AppData,
        context: ViewContext<C>,
        frame: &mut F,
    ) {
        text::StringViewSingleLine::new(Style::new().with_foreground(Rgb24::new_grey(255)))
            .view(self.text, context, frame);
        self.view
            .view(data, context.add_offset(Coord { x: 0, y: 2 }), frame);
    }
}

struct DecorateConfirmMenu(String);
impl Decorate for DecorateConfirmMenu {
    type View = AppView;
    type Data = AppData;
    fn view<E, F, C>(
        &self,
        data: &Self::Data,
        mut event_routine_view: EventRoutineView<E>,
        context: ViewContext<C>,
        frame: &mut F,
    ) where
        E: EventRoutine<Data = Self::Data, View = Self::View>,
        F: Frame,
        C: ColModify,
    {
        if let Some(instance) = data.game.instance() {
            AlignView {
                alignment: Alignment::centre(),
                view: FillBackgroundView {
                    rgb24: colours::SPACE_BACKGROUND.saturating_scalar_mul_div(2, 3),
                    view: BorderView {
                        style: &BorderStyle::new(),
                        view: ConfirmMenuWindow {
                            view: &mut event_routine_view,
                            text: self.0.as_str(),
                        },
                    },
                },
            }
            .view(data, context.add_depth(depth::GAME_MAX + 1), frame);
            event_routine_view.view.game.view(
                GameToRender {
                    game: instance.game(),
                    status: GameStatus::Playing,
                    mouse_coord: None,
                    mode: Mode::Normal,
                    action_error: None,
                },
                context.compose_col_modify(
                    ColModifyDefaultForeground(Rgb24::new_grey(255)).compose(ColModifyMap(
                        |col: Rgb24| col.saturating_scalar_mul_div(1, 3),
                    )),
                ),
                frame,
            );
        }
    }
}

struct DecorateGame;

impl Decorate for DecorateGame {
    type View = AppView;
    type Data = AppData;
    fn view<E, F, C>(
        &self,
        data: &Self::Data,
        mut event_routine_view: EventRoutineView<E>,
        context: ViewContext<C>,
        frame: &mut F,
    ) where
        E: EventRoutine<Data = Self::Data, View = Self::View>,
        F: Frame,
        C: ColModify,
    {
        event_routine_view.view(data, context, frame);
    }
}

struct Quit;

struct MouseTracker<E: EventRoutine> {
    e: E,
}

impl<E: EventRoutine> MouseTracker<E> {
    fn new(e: E) -> Self {
        Self { e }
    }
}

impl<E: EventRoutine<Data = AppData, Event = CommonEvent>> EventRoutine for MouseTracker<E> {
    type Return = E::Return;
    type View = E::View;
    type Data = AppData;
    type Event = CommonEvent;

    fn handle<EP>(
        self,
        data: &mut Self::Data,
        view: &Self::View,
        event_or_peek: EP,
    ) -> Handled<Self::Return, Self>
    where
        EP: EventOrPeek<Event = Self::Event>,
    {
        event_or_peek.with(
            (self, data),
            |(s, data), event| {
                if let CommonEvent::Input(Input::Mouse(MouseInput::MouseMove { coord, .. })) = event
                {
                    data.last_mouse_coord = coord;
                }
                s.e.handle(data, view, event_routine::Event::new(event))
                    .map_continue(|e| Self { e })
            },
            |(s, data)| {
                s.e.handle(data, view, event_routine::Peek::new())
                    .map_continue(|e| Self { e })
            },
        )
    }
    fn view<F, C>(
        &self,
        data: &Self::Data,
        view: &mut Self::View,
        context: ViewContext<C>,
        frame: &mut F,
    ) where
        F: Frame,
        C: ColModify,
    {
        self.e.view(data, view, context, frame)
    }
}

#[derive(Clone, Copy, Debug, PartialOrd, Ord, PartialEq, Eq)]
enum OrBack<T> {
    Selection(T),
    Back,
}

#[derive(Clone, Copy, Debug, PartialOrd, Ord, PartialEq, Eq)]
enum OptionsMenuEntry {
    ToggleMusic,
    ToggleSfx,
    ToggleFullscreen,
}

impl OptionsMenuEntry {
    fn instance(env: &Box<dyn Env>) -> menu::MenuInstanceChooseOrEscape<OrBack<OptionsMenuEntry>> {
        use OptionsMenuEntry::*;
        use OrBack::*;
        menu::MenuInstanceBuilder {
            items: if env.fullscreen_supported() {
                vec![
                    Selection(ToggleMusic),
                    Selection(ToggleSfx),
                    Selection(ToggleFullscreen),
                    Back,
                ]
            } else {
                vec![Selection(ToggleMusic), Selection(ToggleSfx), Back]
            },
            selected_index: 0,
            hotkeys: Some(hashmap![
                'm' => Selection(ToggleMusic),
                's' => Selection(ToggleSfx),
                'f' => Selection(ToggleFullscreen),
            ]),
        }
        .build()
        .unwrap()
        .into_choose_or_escape()
    }
}

struct SelectOptionsMenu;
impl ViewSelector for SelectOptionsMenu {
    type ViewInput = AppView;
    type ViewOutput = FadeMenuInstanceView;
    fn view<'a>(&self, input: &'a Self::ViewInput) -> &'a Self::ViewOutput {
        &input.options_menu
    }
    fn view_mut<'a>(&self, input: &'a mut Self::ViewInput) -> &'a mut Self::ViewOutput {
        &mut input.options_menu
    }
}
impl DataSelector for SelectOptionsMenu {
    type DataInput = AppData;
    type DataOutput = menu::MenuInstanceChooseOrEscape<OrBack<OptionsMenuEntry>>;
    fn data<'a>(&self, input: &'a Self::DataInput) -> &'a Self::DataOutput {
        &input.options_menu
    }
    fn data_mut<'a>(&self, input: &'a mut Self::DataInput) -> &'a mut Self::DataOutput {
        &mut input.options_menu
    }
}
impl Selector for SelectOptionsMenu {}

struct DecorateOptionsMenu;
impl Decorate for DecorateOptionsMenu {
    type View = AppView;
    type Data = AppData;
    fn view<E, F, C>(
        &self,
        data: &Self::Data,
        mut event_routine_view: EventRoutineView<E>,
        context: ViewContext<C>,
        frame: &mut F,
    ) where
        E: EventRoutine<Data = Self::Data, View = Self::View>,
        F: Frame,
        C: ColModify,
    {
        if let Some(instance) = data.game.instance() {
            AlignView {
                alignment: Alignment::centre(),
                view: FillBackgroundView {
                    rgb24: colours::SPACE_BACKGROUND.saturating_scalar_mul_div(2, 3),
                    view: BorderView {
                        style: &BorderStyle::new(),
                        view: &mut event_routine_view,
                    },
                },
            }
            .view(data, context.add_depth(depth::GAME_MAX + 1), frame);
            event_routine_view.view.game.view(
                GameToRender {
                    game: instance.game(),
                    status: GameStatus::Playing,
                    mouse_coord: None,
                    mode: Mode::Normal,
                    action_error: None,
                },
                context.compose_col_modify(
                    ColModifyDefaultForeground(Rgb24::new_grey(255)).compose(ColModifyMap(
                        |col: Rgb24| col.saturating_scalar_mul_div(1, 3),
                    )),
                ),
                frame,
            );
        } else {
            data.menu_background_data.render(context, frame);
            InitMenu(event_routine_view).view(
                &data,
                context.add_offset(Coord { x: 14, y: 24 }).add_depth(100),
                frame,
            );
        }
    }
}

fn options_menu() -> impl EventRoutine<
    Return = Result<OrBack<OptionsMenuEntry>, menu::Escape>,
    Data = AppData,
    View = AppView,
    Event = CommonEvent,
> {
    SideEffectThen::new_with_view(|data: &mut AppData, _: &_| {
        let config = data.game.config();
        let fullscreen = data.env.fullscreen();
        let fullscreen_requires_restart = data.env.fullscreen_requires_restart();
        let menu_entry_string = MenuEntryStringFn::new(
            move |entry: MenuEntryToRender<OrBack<OptionsMenuEntry>>, buf: &mut String| {
                use std::fmt::Write;
                use OptionsMenuEntry::*;
                use OrBack::*;
                match entry.entry {
                    Back => write!(buf, "back").unwrap(),
                    Selection(entry) => match entry {
                        ToggleMusic => write!(
                            buf,
                            "(m) Music enabled [{}]",
                            if config.music { '*' } else { ' ' }
                        )
                        .unwrap(),
                        ToggleSfx => write!(
                            buf,
                            "(s) Sfx enabled [{}]",
                            if config.sfx { '*' } else { ' ' }
                        )
                        .unwrap(),
                        ToggleFullscreen => {
                            if fullscreen_requires_restart {
                                write!(
                                    buf,
                                    "(f) Fullscreen (requires restart) [{}]",
                                    if fullscreen { '*' } else { ' ' }
                                )
                                .unwrap()
                            } else {
                                write!(
                                    buf,
                                    "(f) Fullscreen [{}]",
                                    if fullscreen { '*' } else { ' ' }
                                )
                                .unwrap()
                            }
                        }
                    },
                }
            },
        );
        menu::FadeMenuInstanceRoutine::new(menu_entry_string)
            .select(SelectOptionsMenu)
            .decorated(DecorateOptionsMenu)
            .on_event(|data, event| {
                if let CommonEvent::Frame(since_prev) = event {
                    data.menu_background_data.tick(*since_prev);
                }
            })
    })
}

fn options_menu_cycle(
) -> impl EventRoutine<Return = (), Data = AppData, View = AppView, Event = CommonEvent> {
    make_either!(Ei = A | B);
    use OptionsMenuEntry::*;
    use OrBack::*;
    Ei::A(options_menu()).repeat(|choice| match choice {
        Ok(Back) | Err(menu::Escape) => Handled::Return(()),
        Ok(Selection(selection)) => Handled::Continue(Ei::B(SideEffectThen::new_with_view(
            move |data: &mut AppData, _: &_| {
                let mut config = data.game.config();
                match selection {
                    ToggleMusic => config.music = !config.music,
                    ToggleSfx => config.sfx = !config.sfx,
                    ToggleFullscreen => {
                        data.env.set_fullscreen(!data.env.fullscreen());
                        config.fullscreen = data.env.fullscreen();
                    }
                }
                data.game.set_config(config);
                options_menu()
            },
        ))),
    })
}

#[derive(Clone, Copy)]
pub struct AutoPlay;

#[derive(Clone, Copy)]
pub struct FirstRun;

fn upgrade_menu() -> impl EventRoutine<
    Return = Result<player::Upgrade, menu::Escape>,
    Data = AppData,
    View = AppView,
    Event = CommonEvent,
> {
    make_either!(Ei = A | B);
    SideEffectThen::new_with_view(|data: &mut AppData, _: &_| {
        let upgrades = data.game.instance().unwrap().game().available_upgrades();
        if upgrades.is_empty() {
            Ei::A(Value::new(Err(menu::Escape)))
        } else {
            data.upgrade_menu = Some(
                menu::MenuInstanceBuilder {
                    items: upgrades,
                    selected_index: 0,
                    hotkeys: None,
                }
                .build()
                .unwrap()
                .into_choose_or_escape(),
            );
            let menu = menu::FadeMenuInstanceRoutine::new(MenuEntryStringFn::new(
                |entry: MenuEntryToRender<player::Upgrade>, buf: &mut String| {
                    use std::fmt::Write;
                    let name = match entry.entry.typ {
                        player::UpgradeType::Toughness => "Toughness",
                        player::UpgradeType::Accuracy => "Accuracy",
                        player::UpgradeType::Endurance => "Endurance",
                    };
                    let level = match entry.entry.level {
                        player::UpgradeLevel::Level1 => "1",
                        player::UpgradeLevel::Level2 => "2",
                    };
                    let price = entry.entry.level.cost();
                    write!(buf, "{} {} (${})", name, level, price).unwrap();
                },
            ))
            .select(SelectUpgradeMenu)
            .decorated(DecorateUpgradeMenu);
            Ei::B(menu)
        }
    })
}

fn confirm_menu(
    message: &str,
) -> impl EventRoutine<Return = bool, Data = AppData, View = AppView, Event = CommonEvent> {
    let menu = menu::FadeMenuInstanceRoutine::new(MenuEntryStringFn::new(
        |entry: MenuEntryToRender<bool>, buf: &mut String| {
            use std::fmt::Write;
            if *entry.entry {
                write!(buf, "(y) Yes").unwrap();
            } else {
                write!(buf, "(n) No").unwrap();
            }
        },
    ))
    .select(SelectConfirmMenu)
    .decorated(DecorateConfirmMenu(message.to_string()));
    SideEffectThen::new_with_view(move |data: &mut AppData, _: &_| {
        data.confirm_menu.menu_instance_mut().set_index(0);
        menu.map(|result| result.unwrap_or(false))
    })
}

fn main_menu(
    auto_play: Option<AutoPlay>,
    first_run: Option<FirstRun>,
) -> impl EventRoutine<
    Return = Result<MainMenuEntry, menu::Escape>,
    Data = AppData,
    View = AppView,
    Event = CommonEvent,
> {
    make_either!(Ei = A | B | C | D | E);
    SideEffectThen::new_with_view(move |data: &mut AppData, _: &_| {
        if auto_play.is_some() {
            if first_run.is_some() {
                if data.game.has_instance() {
                    Ei::D(story().map(|()| Ok(MainMenuEntry::Resume)))
                } else {
                    Ei::C(story().map(|()| Ok(MainMenuEntry::NewGame)))
                }
            } else {
                if data.game.has_instance() {
                    Ei::A(Value::new(Ok(MainMenuEntry::Resume)))
                } else {
                    Ei::A(Value::new(Ok(MainMenuEntry::NewGame)))
                }
            }
        } else {
            if data.game.has_instance() {
                match data.main_menu_type {
                    MainMenuType::Init => {
                        data.main_menu =
                            MainMenuEntry::pause(data.frontend).into_choose_or_escape();
                        data.main_menu_type = MainMenuType::Pause;
                    }
                    MainMenuType::Pause => (),
                }
            } else {
                if !data.game.is_music_playing() {
                    data.game.loop_music(Audio::Menu, 0.2);
                }
                if data.game.config().won {
                    data.main_menu = MainMenuEntry::won(data.frontend).into_choose_or_escape();
                    data.main_menu_type = MainMenuType::Init;
                } else {
                    match data.main_menu_type {
                        MainMenuType::Init => (),
                        MainMenuType::Pause => {
                            data.main_menu =
                                MainMenuEntry::init(data.frontend).into_choose_or_escape();
                            data.main_menu_type = MainMenuType::Init;
                        }
                    }
                }
            }
            let menu = menu::FadeMenuInstanceRoutine::new(MenuEntryStringFn::new(
                |entry: MenuEntryToRender<MainMenuEntry>, buf: &mut String| {
                    use std::fmt::Write;
                    let s = match entry.entry {
                        MainMenuEntry::NewGame => "(n) New Game",
                        MainMenuEntry::Resume => "(r) Resume",
                        MainMenuEntry::Quit => "(q) Quit",
                        MainMenuEntry::SaveQuit => "(q) Save and Quit",
                        MainMenuEntry::Save => "(s) Save",
                        MainMenuEntry::Clear => "(c) Clear",
                        MainMenuEntry::Options => "(o) Options",
                        MainMenuEntry::Story => "(p) Prologue",
                        MainMenuEntry::Help => "(h) Help",
                        MainMenuEntry::EndText => "(e) Epilogue",
                    };
                    write!(buf, "{}", s).unwrap();
                },
            ))
            .select(SelectMainMenu)
            .decorated(DecorateMainMenu)
            .on_event(|data, event| {
                if let CommonEvent::Frame(since_prev) = event {
                    data.menu_background_data.tick(*since_prev);
                }
            });
            if first_run.is_some() {
                Ei::E(story().then(|| menu).on_event(|data, event| {
                    if let CommonEvent::Frame(since_prev) = event {
                        data.menu_background_data.tick(*since_prev);
                    }
                }))
            } else {
                Ei::B(menu)
            }
        }
    })
}

fn game(
) -> impl EventRoutine<Return = GameReturn, Data = AppData, View = AppView, Event = CommonEvent> {
    GameEventRoutine::new()
        .select(SelectGame)
        .decorated(DecorateGame)
}

fn game_injecting_inputs(
    inputs: Vec<InjectedInput>,
) -> impl EventRoutine<Return = GameReturn, Data = AppData, View = AppView, Event = CommonEvent> {
    GameEventRoutine::new_injecting_inputs(inputs)
        .select(SelectGame)
        .decorated(DecorateGame)
}

fn game_over() -> impl EventRoutine<Return = (), Data = AppData, View = AppView, Event = CommonEvent>
{
    GameOverEventRoutine::new()
        .select(SelectGame)
        .decorated(DecorateGame)
}

fn choose_weapon_slot() -> impl EventRoutine<
    Return = Option<RangedWeaponSlot>,
    Data = AppData,
    View = AppView,
    Event = CommonEvent,
> {
    ChooseWeaponSlotEventRoutine
        .select(SelectGame)
        .decorated(DecorateGame)
}

fn win_text() -> TextOverlay {
    let bold = Style::new()
        .with_foreground(colours::STRIPE)
        .with_bold(true);
    let normal = Style::new()
        .with_foreground(colours::STRIPE)
        .with_bold(false);
    let faint = Style::new()
        .with_foreground(colours::STRIPE)
        .with_bold(false);
    let t = |text: &str, style| text::RichTextPartOwned::new(text.to_string(), style);
    TextOverlay::new(
        30,
        vec![
            t(
                "With its fuel supply restored, the station flies back into orbit. \
            On autopilot. Shame about the crew, but these things happen. Nobody said \
            space was a safe place to work.\n\n\
            You undock your shuttle and make for Earth. Easy does it. Gotta make it \
            back in one piece and collect on that ",
                normal,
            ),
            t("hefty wager", bold),
            t(
                " you placed on yourself. \
            Serves those suckers right for betting against you!\n\n\
            No doubt there'll be a ton of paperwork to complete before you can go home. \
            The company can't have this getting out. It's gonna be all NDA this and \
            sworn to secrecy that. Don't go running to the press about the ",
                normal,
            ),
            t(
                "undead space \
            station crew",
                bold,
            ),
            t(" you just put down. Now sign here.", normal),
            t("\n\n\n\n\n\nPress any key...", faint),
        ],
    )
}

fn win_text2() -> TextOverlay {
    let bold = Style::new()
        .with_foreground(colours::STRIPE)
        .with_bold(true);
    let normal = Style::new()
        .with_foreground(colours::STRIPE)
        .with_bold(false);
    let faint = Style::new()
        .with_foreground(colours::STRIPE)
        .with_bold(false);
    let t = |text: &str, style| text::RichTextPartOwned::new(text.to_string(), style);
    TextOverlay::new(
        35,
        vec![
            t(
                "Now that you have time to think, something gives you pause. \
            Pretty big coincidence, the station running out of fuel at the ",
                normal,
            ),
            t("same time", bold),
            t(
                " that its crew transforms into a horde of ravenous bloodthirsty monsters. \
                The next scheduled resupply wasn't for months. They should have had plenty \
                of fuel!\n\n\
                And those words in the airlock: \"Don't open! Dead inside!\" Were they meant \
                for you? Who wrote them? How did they know the company would send a shuttle? The airlock was \
                deserted, so whoever wrote it must have gone back inside.\n\n\
                The airlock ",
                normal,
            ),
            t("was", bold),
            t(" empty. Yes. It was empty and you sealed the door behind you. There's no way any of those "
               , normal),
            t("things", bold),
            t(" could have snuck aboard your shuttle.\n\n\
            Everything is fine."
               , normal),
            t("\n\n\n\n\n\nPress any key...", faint),
        ],
    )
}

fn win() -> impl EventRoutine<Return = (), Data = AppData, View = AppView, Event = CommonEvent> {
    SideEffectThen::new_with_view(|data: &mut AppData, _: &_| {
        data.game.loop_music(Audio::EndTextHappy, 0.2);
        let mut config = data.game.config();
        config.won = true;
        data.game.set_config(config);
        win_text()
            .then(|| {
                SideEffectThen::new_with_view(|data: &mut AppData, _: &_| {
                    data.game.loop_music(Audio::EndTextSad, 0.2);
                    win_text2()
                })
            })
            .on_event(|data, event| {
                if let CommonEvent::Frame(since_prev) = event {
                    data.menu_background_data.tick(*since_prev);
                }
            })
    })
}

fn cannot_afford() -> TextOverlay {
    let normal = Style::new()
        .with_foreground(colours::STRIPE)
        .with_bold(false);
    let t = |text: &str, style| text::RichTextPartOwned::new(text.to_string(), style);
    TextOverlay::new(1, vec![t("You can't afford that!", normal)])
}

fn story() -> TextOverlay {
    let bold = Style::new()
        .with_foreground(colours::STRIPE)
        .with_bold(true);
    let normal = Style::new()
        .with_foreground(colours::STRIPE)
        .with_bold(false);
    let faint = Style::new()
        .with_foreground(colours::STRIPE)
        .with_bold(false);
    let t = |text: &str, style| text::RichTextPartOwned::new(text.to_string(), style);
    TextOverlay::new(40, vec![
        t("You tape over the flashing warning light. An overheating engine is the least of your worries. \
        Gotta focus.\n\n\
        The space station looms ahead. It's out of fuel, and about to come crashing down to Earth. \
        Unless you get to it first. \
        Special delivery: 1 hydrogen fuel cell with enough juice to kick the station out of this pesky \
        atmosphere and back into space where it belongs.\n\n\
        Back home your buddies are placing bets on whether you'll make it back alive. \
        Last you heard, odds were 5 to 1 against.\n\n\
        \"Docking complete,\" sounds a lifeless mechanical voice. No word yet from the station crew. Comms must be down. Figures. \
        Shouldering your pack containing the fuel cell, you trudge into the airlock. \
        Gotta lug this thing down the five flights of stairs to the fuel bay. Who designed this place?\n\n\
        A dim light flickers on in the airlock revealing words smeared in blood on the opposite door:\n", normal),
        t("DON'T OPEN! DEAD INSIDE!", bold),
        t("\n\n\
        Better make those odds 6 to 1...", normal),
        t("\n\n\n\n\n\nPress any key...", faint),
    ])
}

fn help() -> TextOverlay {
    let normal = Style::new()
        .with_foreground(colours::STRIPE)
        .with_bold(false);
    let faint = Style::new().with_foreground(colours::STRIPE);
    let b = |s: &str| text::RichTextPartOwned::new(s.to_string(), normal.with_bold(true));
    let t = |s: &str| text::RichTextPartOwned::new(s.to_string(), normal);
    TextOverlay::new(
        52,
        vec![
            b("Combat\n"),
            t("Each weapon has a DMG(♥) and PEN(♦) stat, and each enemy has heatlh(♥) and armour(♦). "),
            t("If an enemy is hit with a weapon that has a higher PEN than their armour, their health is "),
            t("reduced by the weapon's DMG. If a projectile's PEN exceeds an enemy's armour, it continues "),
            t("on its path with its PEN reduced by the enemy's armour. If it hits the hull, it has a "),
            t("chance to breach the hull (its HULL PEN stat).\n\n"),
            b("Hull Breaches\n"),
            t("If the hull is breached the air is sucked out of connected areas of the station. "),
            t("For several turns, characters and items in connected areas are pulled towards the breach. "),
            t("This is indicated by a red light. "),
            t("After the air is drained, the light will turn blue indicating vacuum. "),
            t("Your oxygen will start decreasing, and if it runs out then your health will start decreasing "),
            t("until you get back into a pressurised area.\n\n"),
            b("Keyboard Controls\n"),
            t("Movement/Aim: Arrows/WASD/HJKL\n"),
            t("Cancel Aim: Escape\n"),
            t("Wait: Space\n"),
            t("Examine: X\n"),
            t("Get Weapon: G\n"),
            t("Fire Ranged Weapon: 1-3\n\n"),
            b("Gamepad Controls\n"),
            t("Movement/Aim: D-Pad\n"),
            t("Cancel Aim: Select\n"),
            t("Wait: Select\n"),
            t("Examine: Right Bumper\n"),
            t("Get Weapon: Y/Triangle\n"),
            t("Fire Ranged Weapon Slot 1: X/Square\n"),
            t("Fire Ranged Weapon Slot 2: A/Cross\n"),
            t("Fire Ranged Weapon Slot 2: B/Circle\n"),
            text::RichTextPartOwned::new("\n\n\n\n\nPress any key...".to_string(), faint),
        ],
    )
}

fn aim(
    slot: RangedWeaponSlot,
) -> impl EventRoutine<Return = Option<Fire>, Data = AppData, View = AppView, Event = CommonEvent> {
    AimEventRoutine::new(slot)
        .select(SelectGame)
        .decorated(DecorateGame)
}

fn examine() -> impl EventRoutine<Return = (), Data = AppData, View = AppView, Event = CommonEvent>
{
    make_either!(Ei = A | B);
    SideEffectThen::new_with_view(|data: &mut AppData, _view: &AppView| {
        let game_relative_mouse_coord = ScreenCoord(data.last_mouse_coord);
        if let Ok(initial_aim_coord) = data.game.initial_aim_coord(game_relative_mouse_coord) {
            Ei::A(
                ExamineEventRoutine::new(initial_aim_coord.0)
                    .select(SelectGame)
                    .decorated(DecorateGame),
            )
        } else {
            Ei::B(Value::new(()))
        }
    })
}

enum GameLoopBreak {
    GameOver,
    Win,
    Pause,
}

fn game_loop() -> impl EventRoutine<Return = (), Data = AppData, View = AppView, Event = CommonEvent>
{
    make_either!(Ei = A | B | C | D | E | F | G);
    SideEffect::new_with_view(|data: &mut AppData, _: &_| data.game.pre_game_loop())
        .then(|| {
            Ei::A(game())
                .repeat(|game_return| match game_return {
                    GameReturn::Examine => {
                        Handled::Continue(Ei::C(examine().and_then(|()| game())))
                    }
                    GameReturn::Pause => Handled::Return(GameLoopBreak::Pause),
                    GameReturn::GameOver => Handled::Return(GameLoopBreak::GameOver),
                    GameReturn::Win => Handled::Return(GameLoopBreak::Win),
                    GameReturn::Aim(slot) => {
                        Handled::Continue(Ei::B(aim(slot).and_then(|maybe_fire| {
                            make_either!(Ei = A | B);
                            if let Some(fire) = maybe_fire {
                                Ei::A(game_injecting_inputs(vec![InjectedInput::Fire(fire)]))
                            } else {
                                Ei::B(game())
                            }
                        })))
                    }
                    GameReturn::Upgrade => {
                        Handled::Continue(Ei::D(upgrade_menu().and_then(|upgrade| {
                            make_either!(Ei = A | B);
                            if let Ok(upgrade) = upgrade {
                                Ei::A(SideEffectThen::new_with_view(
                                    move |data: &mut AppData, _: &_| {
                                        make_either!(Ei = A | B);
                                        if upgrade.level.cost()
                                            > data.game.instance().unwrap().game().player().credit
                                        {
                                            Ei::A(cannot_afford().then(|| game()))
                                        } else {
                                            Ei::B(game_injecting_inputs(vec![
                                                InjectedInput::Upgrade(upgrade),
                                            ]))
                                        }
                                    },
                                ))
                            } else {
                                Ei::B(game())
                            }
                        })))
                    }
                    GameReturn::EquipRanged => {
                        Handled::Continue(Ei::E(choose_weapon_slot().and_then(|slot| {
                            SideEffectThen::new_with_view(move |data: &mut AppData, _: &_| {
                                make_either!(Ei = A | B | C);
                                if let Some(slot) = slot {
                                    if data
                                        .game
                                        .instance()
                                        .unwrap()
                                        .game()
                                        .player_has_weapon_in_slot(slot)
                                    {
                                        Ei::A(
                                            confirm_menu(
                                                format!(
                                                    "Replace ranged weapon in slot {}?",
                                                    slot.index() + 1
                                                )
                                                .as_str(),
                                            )
                                            .and_then(
                                                move |yes| {
                                                    let inputs = if yes {
                                                        vec![InjectedInput::GetRangedWeapon(slot)]
                                                    } else {
                                                        vec![]
                                                    };
                                                    game_injecting_inputs(inputs)
                                                },
                                            ),
                                        )
                                    } else {
                                        Ei::B(game_injecting_inputs(vec![
                                            InjectedInput::GetRangedWeapon(slot),
                                        ]))
                                    }
                                } else {
                                    Ei::C(game())
                                }
                            })
                        })))
                    }
                    GameReturn::ConfirmReplaceMelee => Handled::Continue(Ei::F(
                        confirm_menu("Replace current melee weapon?").and_then(|yes| {
                            let inputs = if yes {
                                vec![InjectedInput::GetMeleeWeapon]
                            } else {
                                vec![]
                            };
                            game_injecting_inputs(inputs)
                        }),
                    )),
                    GameReturn::UnlockMap => Handled::Continue(Ei::G(
                        confirm_menu("Spend $2 credit to unlock map?").and_then(|yes| {
                            make_either!(Ei = A | B);
                            if yes {
                                Ei::A(SideEffectThen::new_with_view(
                                    move |data: &mut AppData, _: &_| {
                                        make_either!(Ei = A | B);
                                        if data.game.instance().unwrap().game().player().credit < 2
                                        {
                                            Ei::A(cannot_afford().then(|| game()))
                                        } else {
                                            Ei::B(game_injecting_inputs(vec![
                                                InjectedInput::UnlockMap,
                                            ]))
                                        }
                                    },
                                ))
                            } else {
                                Ei::B(game())
                            }
                        }),
                    )),
                })
                .and_then(|game_loop_break| {
                    make_either!(Ei = A | B | C);
                    match game_loop_break {
                        GameLoopBreak::Win => Ei::C(SideEffectThen::new_with_view(
                            |data: &mut AppData, _: &_| {
                                data.game.clear_instance();
                                win().on_event(|data, event| {
                                    if let CommonEvent::Frame(since_prev) = event {
                                        data.menu_background_data.tick(*since_prev);
                                    }
                                })
                            },
                        )),
                        GameLoopBreak::Pause => Ei::A(Value::new(())),
                        GameLoopBreak::GameOver => Ei::B(game_over().and_then(|()| {
                            SideEffect::new_with_view(|data: &mut AppData, _: &_| {
                                data.game.clear_instance();
                            })
                        })),
                    }
                })
        })
        .then(|| SideEffect::new_with_view(|data: &mut AppData, _: &_| data.game.post_game_loop()))
}

fn main_menu_cycle(
    auto_play: Option<AutoPlay>,
    first_run: Option<FirstRun>,
) -> impl EventRoutine<Return = Option<Quit>, Data = AppData, View = AppView, Event = CommonEvent> {
    make_either!(Ei = A | B | C | D | E | F | G | H | I | J);
    main_menu(auto_play, first_run).and_then(|entry| match entry {
        Ok(MainMenuEntry::Quit) => Ei::A(Value::new(Some(Quit))),
        Ok(MainMenuEntry::SaveQuit) => {
            Ei::D(SideEffect::new_with_view(|data: &mut AppData, _: &_| {
                data.game.save_instance();
                Some(Quit)
            }))
        }
        Ok(MainMenuEntry::Save) => Ei::E(SideEffectThen::new_with_view(
            |data: &mut AppData, _: &_| {
                make_either!(Ei = A | B);
                data.game.save_instance();
                if data.game.has_instance() {
                    Ei::A(game_loop().map(|_| None))
                } else {
                    Ei::B(Value::new(None))
                }
            },
        )),
        Ok(MainMenuEntry::Clear) => {
            Ei::F(SideEffect::new_with_view(|data: &mut AppData, _: &_| {
                data.game.clear_instance();
                None
            }))
        }
        Ok(MainMenuEntry::Resume) | Err(menu::Escape) => Ei::B(SideEffectThen::new_with_view(
            |data: &mut AppData, _: &_| {
                make_either!(Ei = A | B);
                if data.game.has_instance() {
                    Ei::A(game_loop().map(|()| None))
                } else {
                    Ei::B(Value::new(None))
                }
            },
        )),
        Ok(MainMenuEntry::NewGame) => Ei::C(SideEffectThen::new_with_view(
            |data: &mut AppData, _: &_| {
                data.game.instantiate();
                data.main_menu.menu_instance_mut().set_index(0);
                game_loop().map(|()| None)
            },
        )),
        Ok(MainMenuEntry::Options) => Ei::G(options_menu_cycle().map(|_| None)),
        Ok(MainMenuEntry::Story) => Ei::H(story().map(|()| None).on_event(|data, event| {
            if let CommonEvent::Frame(since_prev) = event {
                data.menu_background_data.tick(*since_prev);
            }
        })),
        Ok(MainMenuEntry::Help) => Ei::I(help().map(|()| None).on_event(|data, event| {
            if let CommonEvent::Frame(since_prev) = event {
                data.menu_background_data.tick(*since_prev);
            }
        })),
        Ok(MainMenuEntry::EndText) => Ei::J(
            SideEffectThen::new_with_view(|data: &mut AppData, _: &_| {
                data.game.loop_music(Audio::EndTextHappy, 0.2);
                win_text()
            })
            .then(|| {
                SideEffectThen::new_with_view(|data: &mut AppData, _: &_| {
                    data.game.loop_music(Audio::EndTextSad, 0.2);
                    win_text2()
                })
            })
            .map(|()| None)
            .on_event(|data, event| {
                if let CommonEvent::Frame(since_prev) = event {
                    data.menu_background_data.tick(*since_prev);
                }
            }),
        ),
    })
}

struct PrimeFont;
impl EventRoutine for PrimeFont {
    type Return = ();
    type Data = AppData;
    type View = AppView;
    type Event = CommonEvent;
    fn handle<EP>(
        self,
        data: &mut Self::Data,
        _view: &Self::View,
        event_or_peek: EP,
    ) -> Handled<Self::Return, Self>
    where
        EP: EventOrPeek<Event = Self::Event>,
    {
        event_or_peek_with_handled(event_or_peek, self, |s, event| match event {
            CommonEvent::Input(_) => Handled::Continue(s),
            CommonEvent::Frame(duration) => {
                if let Some(remaining) = data.prime_font_countdown.checked_sub(duration) {
                    data.prime_font_countdown = remaining;
                    Handled::Continue(s)
                } else {
                    Handled::Return(())
                }
            }
        })
    }
    fn view<F, C>(
        &self,
        _data: &Self::Data,
        _view: &mut Self::View,
        context: ViewContext<C>,
        frame: &mut F,
    ) where
        F: Frame,
        C: ColModify,
    {
        let string = "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ1234567890'\"/\\+=-_~`.,-!@#$%^&*()♥♦{}[]▄▀▗▖▝▘▐▌►:; ●?──││┌┐└┘┤├┬";
        let text = vec![
            text::RichTextPart::new(
                string,
                Style::new()
                    .with_foreground(Rgb24::new_grey(0))
                    .with_bold(false),
            ),
            text::RichTextPart::new(
                string,
                Style::new()
                    .with_foreground(Rgb24::new_grey(0))
                    .with_bold(true),
            ),
        ];
        text::RichTextView::new(text::wrap::Char::new()).view(text, context, frame);
    }
}

fn event_routine(
    initial_auto_play: Option<AutoPlay>,
) -> impl EventRoutine<Return = (), Data = AppData, View = AppView, Event = CommonEvent> {
    MouseTracker::new(SideEffectThen::new_with_view(
        move |data: &mut AppData, _: &_| {
            let mut config = data.game.config();
            let first_run = config.first_run;
            config.first_run = false;
            data.game.set_config(config);
            let first_run = if first_run { Some(FirstRun) } else { None };
            main_menu_cycle(initial_auto_play, first_run)
                .repeat(|maybe_quit| {
                    if let Some(Quit) = maybe_quit {
                        Handled::Return(())
                    } else {
                        Handled::Continue(main_menu_cycle(None, None))
                    }
                })
                .return_on_exit(|data| {
                    data.game.save_instance();
                    ()
                })
        },
    ))
}

pub trait Env {
    fn fullscreen(&self) -> bool;
    fn fullscreen_requires_restart(&self) -> bool;
    fn fullscreen_supported(&self) -> bool;
    // hack to get around fact that changing fullscreen mid-game on windows crashes
    fn set_fullscreen_init(&self, fullscreen: bool);
    fn set_fullscreen(&self, fullscreen: bool);
}
pub struct EnvNull;
impl Env for EnvNull {
    fn fullscreen(&self) -> bool {
        false
    }
    fn fullscreen_requires_restart(&self) -> bool {
        false
    }
    fn fullscreen_supported(&self) -> bool {
        false
    }
    fn set_fullscreen(&self, _fullscreen: bool) {}
    fn set_fullscreen_init(&self, _fullscreen: bool) {}
}

pub struct Fullscreen;

pub fn app(
    game_config: GameConfig,
    frontend: Frontend,
    controls: Controls,
    storage: StaticStorage,
    save_key: String,
    audio_player: AppAudioPlayer,
    rng_seed: RngSeed,
    auto_play: Option<AutoPlay>,
    fullscreen: Option<Fullscreen>,
    env: Box<dyn Env>,
) -> impl app::App {
    let app_data = AppData::new(
        game_config,
        frontend,
        controls,
        storage,
        save_key,
        audio_player,
        rng_seed,
        fullscreen,
        env,
    );
    let app_view = AppView::new();
    PrimeFont
        .then(move || event_routine(auto_play))
        .app_one_shot_ignore_return(app_data, app_view)
}
