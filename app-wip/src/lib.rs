use chargrid::{control_flow::*, core::*};
use orbital_decay_game::Config;

mod colours;
mod controls;
mod game;
mod game_instance;
mod game_loop;
mod menu_background;
mod stars;
mod text;
mod tile_3x3;

pub use game_loop::{InitialRngSeed, SaveGameStorage};

struct AppState {
    game_loop_state: game_loop::GameLoopData,
}

pub struct AppArgs {
    pub save_game_storage: SaveGameStorage,
    pub initial_rng_seed: InitialRngSeed,
    pub omniscient: bool,
}

pub fn app(
    AppArgs {
        save_game_storage,
        initial_rng_seed,
        omniscient,
    }: AppArgs,
) -> impl Component<Output = app::Output, State = ()> {
    let config = Config {
        omniscient: if omniscient { Config::OMNISCIENT } else { None },
        demo: false,
        debug: true,
    };
    let (game_loop_state, initial_state) =
        game_loop::GameLoopData::new(config, save_game_storage, initial_rng_seed);
    let state = AppState { game_loop_state };
    game_loop::game_loop_component(initial_state)
        .lens_state(lens!(AppState[game_loop_state]: game_loop::GameLoopData))
        .map(|_| app::Exit)
        .with_state(state)
        .clear_each_frame()
        .exit_on_close()
}
