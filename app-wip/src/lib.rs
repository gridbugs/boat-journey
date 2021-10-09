use chargrid::{control_flow::*, core::*};
use orbital_decay_game::Config;
use rand::SeedableRng;
use rand_isaac::Isaac64Rng;

mod colours;
mod controls;
mod game;
mod game_loop;
mod stars;
mod tile_3x3;

pub use game_loop::{RngSeed, SaveGameStorage};

struct AppState {
    game_loop_state: game_loop::GameLoopData,
}

pub struct AppArgs {
    pub save_game_storage: SaveGameStorage,
    pub rng_seed: RngSeed,
}

pub fn app(
    AppArgs {
        save_game_storage,
        rng_seed,
    }: AppArgs,
) -> impl Component<Output = app::Output, State = ()> {
    let config = Config {
        omniscient: Config::OMNISCIENT,
        demo: false,
        debug: true,
    };
    let rng = Isaac64Rng::from_entropy();
    let (game_loop_state, running) = game_loop::GameLoopData::new(config, save_game_storage, rng);
    let state = AppState { game_loop_state };
    game_loop::game_loop_component(running)
        .lens_state(lens!(AppState[game_loop_state]: game_loop::GameLoopData))
        .map(|_| app::Exit)
        .with_state(state)
        .clear_each_frame()
        .exit_on_close()
}
