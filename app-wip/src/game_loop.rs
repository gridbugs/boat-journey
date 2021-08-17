use crate::{
    controls::{AppInput, Controls},
    game,
    menu::*,
    stars::Stars,
};
use chargrid::{control_flow::*, input::*, prelude::*};
use orbital_decay_game::{
    player,
    witness::{self, Witness},
    Config, Game,
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
            .render_with_visibility(self.game.visibility_grid(), ctx, fb);
        game::render_game(&self.game, ctx, fb);
    }
}

impl GameLoopState {
    pub fn new<R: Rng>(config: Config, rng: &mut R) -> (Self, witness::Running) {
        let (game, running) = Game::witness_new(&config, rng);
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

impl Component for GameInstanceComponent {
    type Output = Option<Witness>;
    type State = GameLoopState;
    fn render(&self, state: &Self::State, ctx: Ctx, fb: &mut FrameBuffer) {
        state.render(ctx, fb);
    }

    fn update(&mut self, state: &mut Self::State, _ctx: Ctx, event: Event) -> Self::Output {
        let running = self.0.take().unwrap();
        let witness = match event {
            Event::Input(input) => match input {
                Input::Keyboard(keyboard_input) => {
                    if let Some(app_input) = state.controls.get(keyboard_input) {
                        let (witness, action_result) = match app_input {
                            AppInput::Move(direction) => {
                                state.game.witness_walk(direction, &state.config, running)
                            }
                            AppInput::Wait => state.game.witness_wait(&state.config, running),
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
                _ => Witness::Running(running),
            },
            Event::Tick(since_previous) => {
                state
                    .game
                    .witness_tick(since_previous, &state.config, running)
            }
            _ => Witness::Running(running),
        };
        match witness {
            Witness::Running(running) => {
                self.0 = Some(running);
                None
            }
            other => Some(other),
        }
    }

    fn size(&self, _state: &Self::State, ctx: Ctx) -> Size {
        ctx.bounding_box.size()
    }
}

fn upgrade_component(
    upgrade: witness::Upgrade,
) -> CF<impl Component<State = GameLoopState, Output = Option<Witness>>> {
    val_once(Witness::debug_running())
}

fn game_instance_component(
    running: witness::Running,
) -> CF<impl Component<State = GameLoopState, Output = Option<Witness>>> {
    cf(GameInstanceComponent(Some(running)))
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
