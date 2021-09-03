use crate::{
    controls::{AppInput, Controls},
    game,
    stars::Stars,
};
use chargrid::{border::BorderStyle, control_flow::*, input::*, menu, prelude::*};
use orbital_decay_game::{
    player,
    witness::{self, Game, Witness},
    Config,
};
use rand::Rng;

pub struct GameLoopState {
    game: Game,
    stars: Stars,
    controls: Controls,
    config: Config,
}

impl GameLoopState {
    fn render(&self, ctx: Ctx, fb: &mut FrameBuffer) {
        self.stars
            .render_with_visibility(self.game.inner_ref().visibility_grid(), ctx, fb);
        game::render_game(self.game.inner_ref(), ctx, fb);
    }
    fn has_game_ever_been_won(&self) -> bool {
        // TODO
        true
    }
    fn update(&mut self, event: Event, running: witness::Running) -> Witness {
        match event {
            Event::Input(Input::Keyboard(keyboard_input)) => {
                if let Some(app_input) = self.controls.get(keyboard_input) {
                    let (witness, action_result) = match app_input {
                        AppInput::Move(direction) => {
                            running.walk(&mut self.game, direction, &self.config)
                        }
                        AppInput::Wait => running.wait(&mut self.game, &self.config),
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
                running.tick(&mut self.game, since_previous, &self.config)
            }
            _ => Witness::Running(running),
        }
    }
}

impl GameLoopState {
    pub fn new<R: Rng>(config: Config, rng: &mut R) -> (Self, witness::Running) {
        let (game, running) = witness::new_game(&config, rng);
        let stars = Stars::new(rng);
        let controls = Controls::default();
        (
            Self {
                game,
                stars,
                controls,
                config,
            },
            running,
        )
    }
}

struct GameInstanceComponent(Option<witness::Running>);

enum AppState {
    Paused(witness::Running),
    Playing(Witness),
}

impl Component for GameInstanceComponent {
    type Output = Witness;
    type State = GameLoopState;
    fn render(&self, state: &Self::State, ctx: Ctx, fb: &mut FrameBuffer) {
        state.render(ctx, fb);
    }

    fn update(&mut self, state: &mut Self::State, _ctx: Ctx, event: Event) -> Self::Output {
        let running = self.0.take().unwrap();
        if event.is_escape() {
            Witness::Running(running)
        } else {
            state.update(event, running)
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
) -> CF<impl Component<State = GameLoopState, Output = Option<player::Upgrade>>> {
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
            render_state(|state: &GameLoopState, ctx, fb| state.render(ctx, fb)),
            chargrid::core::TintDim(63),
            10,
        )
}

fn upgrade_component(
    upgrade_witness: witness::Upgrade,
) -> CF<impl Component<State = GameLoopState, Output = Option<Witness>>> {
    on_state_then(|state: &mut GameLoopState| {
        let upgrades = state.game.inner_ref().available_upgrades();
        upgrade_menu(upgrades).catch_escape().and_then(|result| {
            on_state(move |state: &mut GameLoopState| {
                let (witness, result) = match result {
                    Err(Escape) => upgrade_witness.cancel(),
                    Ok(upgrade) => upgrade_witness.upgrade(&mut state.game, upgrade, &state.config),
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
enum PauseMenuEntry {
    Resume,
    SaveQuit,
    NewGame,
    Options,
    Help,
    Prologue,
    Epilogue,
    Clear,
}

fn pause_menu() -> CF<impl Component<State = GameLoopState, Output = Option<PauseMenuEntry>>> {
    on_state_then(|state: &mut GameLoopState| {
        use menu::builder::*;
        use PauseMenuEntry::*;
        let mut builder = menu_builder();
        let mut add_item = |entry, name, ch: char| {
            builder.add_item_mut(
                item(Resume, identifier::simple(&format!("({}) {}", ch, name))).add_hotkey_char(ch),
            );
        };
        add_item(Resume, "Resume", 'r');
        add_item(SaveQuit, "Save and Quit", 'q');
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
                render_state(|state: &GameLoopState, ctx, fb| state.render(ctx, fb)),
                chargrid::core::TintDim(63),
                10,
            )
    })
}

fn game_instance_component(
    running: witness::Running,
) -> CF<impl Component<State = GameLoopState, Output = Option<Witness>>> {
    either!(Ei = A | B);
    cf(GameInstanceComponent(Some(running)))
        .some()
        .catch_escape()
        .and_then(|or_escape| match or_escape {
            Ok(witness) => Ei::A(val_once(witness)),
            Err(Escape) => Ei::B(never()),
        })
}

pub enum GameExitReason {
    GameOver,
}

pub fn game_loop_component(
    running: witness::Running,
) -> CF<impl Component<State = GameLoopState, Output = Option<GameExitReason>>> {
    loop_(Witness::Running(running), |witness| {
        either!(Ei = A | B | C);
        match witness {
            Witness::Running(running) => Ei::A(game_instance_component(running).continue_()),
            Witness::Upgrade(upgrade) => Ei::B(upgrade_component(upgrade).continue_()),
            Witness::GameOver => Ei::C(val_once(GameExitReason::GameOver).break_()),
        }
    })
}
