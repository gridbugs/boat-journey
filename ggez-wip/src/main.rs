use chargrid_ggez::*;
use orbital_decay_app_wip::{app, AppArgs};
use orbital_decay_native_wip::NativeCommon;

const CELL_SIZE: f64 = 12.;

fn main() {
    env_logger::init();
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
    let NativeCommon { save_game_storage } = NativeCommon::new();
    context.run(app(AppArgs { save_game_storage }));
}
