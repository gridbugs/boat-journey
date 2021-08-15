use crate::{
    controls::{AppInput, Controls},
    game::{self, GameOutput},
    stars::Stars,
};
use chargrid::{control_flow::*, input::*, prelude::*};
use orbital_decay_game::{Config, Game, Input as GameInput};
use rand::Rng;

pub struct GameInstanceState {
    game: Game,
    stars: Stars,
    controls: Controls,
    config: Config,
}

impl GameInstanceState {
    pub fn new<R: Rng>(config: Config, rng: &mut R) -> Self {
        let game = Game::new(&config, rng);
        let stars = Stars::new(rng);
        let controls = Controls::default();
        Self {
            game,
            stars,
            controls,
            config,
        }
    }
}

pub struct GameInstanceComponent;

impl Component for GameInstanceComponent {
    type Output = Option<GameOutput>;
    type State = GameInstanceState;
    fn render(&self, state: &Self::State, ctx: Ctx, fb: &mut FrameBuffer) {
        state
            .stars
            .render_with_visibility(state.game.visibility_grid(), ctx, fb);
        game::render_game(&state.game, ctx, fb);
    }

    fn update(&mut self, state: &mut Self::State, _ctx: Ctx, event: Event) -> Self::Output {
        let maybe_control_flow = match event {
            Event::Input(input) => match input {
                Input::Keyboard(keyboard_input) => {
                    if let Some(app_input) = state.controls.get(keyboard_input) {
                        let result_control_flow = match app_input {
                            AppInput::Move(direction) => state
                                .game
                                .handle_input(GameInput::Walk(direction), &state.config),
                            AppInput::Wait => {
                                state.game.handle_input(GameInput::Wait, &state.config)
                            }
                            AppInput::Examine | AppInput::Aim(_) | AppInput::Get => {
                                println!("todo");
                                Ok(None)
                            }
                        };
                        match result_control_flow {
                            Ok(maybe_control_flow) => maybe_control_flow,
                            Err(err) => {
                                println!("action error: {:?}", err);
                                None
                            }
                        }
                    } else {
                        None
                    }
                }
                _ => None,
            },
            Event::Tick(since_previous) => state.game.handle_tick(since_previous, &state.config),
            _ => None,
        };
        if let Some(control_flow) = maybe_control_flow {
            panic!("todo: {:?}", control_flow);
        }
        None
    }

    fn size(&self, _state: &Self::State, ctx: Ctx) -> Size {
        ctx.bounding_box.size()
    }
}

pub fn game_instance_component(
) -> CF<impl Component<State = GameInstanceState, Output = Option<GameOutput>>> {
    cf(GameInstanceComponent)
}
