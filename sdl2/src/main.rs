#![windows_subsystem = "windows"]
use boat_journey_app::{app, AppArgs};
use boat_journey_native::{meap, NativeCommon};
use gridbugs::chargrid_sdl2::*;

fn main() {
    use meap::Parser;
    env_logger::init();
    let NativeCommon {
        storage,
        initial_rng_seed,
        audio_player,
        omniscient,
        new_game,
    } = NativeCommon::parser()
        .with_help_default()
        .parse_env_or_exit();
    let context = Context::new(Config {
        font_bytes: FontBytes {
            normal: include_bytes!("./fonts/PxPlus_IBM_CGAthin-with-quadrant-blocks.ttf").to_vec(),
            bold: include_bytes!("./fonts/PxPlus_IBM_CGA-with-quadrant-blocks.ttf").to_vec(),
        },
        title: "Boat Journey".to_string(),
        window_dimensions_px: Dimensions {
            width: 960.,
            height: 720.,
        },
        cell_dimensions_px: Dimensions {
            width: 12.,
            height: 12.,
        },
        font_point_size: 12,
        character_cell_offset: Dimensions {
            width: 0.,
            height: -1.,
        },
        underline_width_cell_ratio: 0.1,
        underline_top_offset_cell_ratio: 0.8,
        resizable: false,
    });
    context.run(app(AppArgs {
        storage,
        initial_rng_seed,
        audio_player,
        omniscient,
        new_game,
    }));
}
