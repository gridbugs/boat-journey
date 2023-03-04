use gridbugs::chargrid::{control_flow::*, core::*};
use template2023_game::Config;

mod audio;
mod colours;
mod controls;
mod examine;
mod game;
mod game_instance;
mod game_loop;
mod menu_background;
mod stars;
mod text;
mod tile_3x3;
mod ui;

pub use audio::AppAudioPlayer;
pub use game_loop::{AppStorage, InitialRngSeed};

struct AppState {
    game_loop_data: game_loop::GameLoopData,
}

pub struct AppArgs {
    pub storage: AppStorage,
    pub initial_rng_seed: InitialRngSeed,
    pub audio_player: AppAudioPlayer,
    pub omniscient: bool,
    pub new_game: bool,
}

pub fn app(
    AppArgs {
        storage,
        initial_rng_seed,
        audio_player,
        omniscient,
        new_game,
    }: AppArgs,
) -> impl Component<Output = app::Output, State = ()> {
    let config = Config {
        omniscient: if omniscient { Config::OMNISCIENT } else { None },
        demo: false,
        debug: false,
    };
    let (game_loop_data, initial_state) =
        game_loop::GameLoopData::new(config, storage, initial_rng_seed, audio_player, new_game);
    let state = AppState { game_loop_data };
    game_loop::game_loop_component(initial_state)
        .lens_state(lens!(AppState[game_loop_data]: game_loop::GameLoopData))
        .map(|_| app::Exit)
        .with_state(state)
        .clear_each_frame()
        .exit_on_close()
}
