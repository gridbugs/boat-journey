use chargrid::{control_flow::*, core::*};
use orbital_decay_game::Config;
use rand::SeedableRng;
use rand_isaac::Isaac64Rng;

mod colours;
mod controls;
mod game;
mod game_instance;
mod stars;
mod tile_3x3;

struct AppState {
    game_instance_state: game_instance::GameInstanceState,
}

pub fn app() -> impl Component<Output = app::Output, State = ()> {
    let config = Config {
        omniscient: None,
        demo: false,
        debug: true,
    };
    let mut rng = Isaac64Rng::from_entropy();
    let game_instance_state = game_instance::GameInstanceState::new(config, &mut rng);
    let state = AppState {
        game_instance_state,
    };
    loop_state(state, || {
        game_instance::game_instance_component()
            .lens_state(lens!(
                AppState[game_instance_state]: game_instance::GameInstanceState
            ))
            .map(|_| LoopControl::Break(app::Exit))
    })
    .clear_each_frame()
    .exit_on_close()
}
