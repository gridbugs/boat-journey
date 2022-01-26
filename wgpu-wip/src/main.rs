use chargrid_wgpu::*;
use orbital_decay_app_wip::{app, AppArgs};
use orbital_decay_native_wip::{meap, NativeCommon};

const CELL_SIZE: f64 = 12.;

fn main() {
    use meap::Parser;
    env_logger::init();
    let NativeCommon {
        storage,
        initial_rng_seed,
        audio_player,
        omniscient,
    } = NativeCommon::parser()
        .with_help_default()
        .parse_env_or_exit();
    let context = Context::new(Config {
        font_bytes: FontBytes {
            normal: include_bytes!("./fonts/PxPlus_IBM_CGAthin-with-quadrant-blocks.ttf").to_vec(),
            bold: include_bytes!("./fonts/PxPlus_IBM_CGA-with-quadrant-blocks.ttf").to_vec(),
        },
        title: "Orbital Decay".to_string(),
        window_dimensions_px: Dimensions {
            width: 960.,
            height: 720.,
        },
        cell_dimensions_px: Dimensions {
            width: CELL_SIZE,
            height: CELL_SIZE,
        },
        font_scale: Dimensions {
            width: CELL_SIZE,
            height: CELL_SIZE,
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
    }));
}
