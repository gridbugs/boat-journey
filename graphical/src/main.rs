#![windows_subsystem = "windows"]
use chargrid_graphical as graphical;
use orbital_decay_app::{app, Frontend, Fullscreen};
use orbital_decay_native::{meap, NativeCommon};

const FULLSCREEN_SUPPORTED: bool = true;

const CELL_SIZE: f64 = 12.;

#[cfg(target_os = "windows")]
mod graphical_env {
    use super::graphical::WindowHandle;
    use orbital_decay_app::Env;
    use std::cell::RefCell;
    pub struct GraphicalEnv {
        window_handle: WindowHandle,
        shadow_fullscreen: RefCell<bool>,
    }
    impl GraphicalEnv {
        pub fn new(window_handle: WindowHandle) -> Self {
            Self {
                window_handle,
                shadow_fullscreen: RefCell::new(false),
            }
        }
    }
    impl Env for GraphicalEnv {
        fn fullscreen(&self) -> bool {
            *self.shadow_fullscreen.borrow()
        }
        fn fullscreen_requires_restart(&self) -> bool {
            true
        }
        fn fullscreen_supported(&self) -> bool {
            super::FULLSCREEN_SUPPORTED
        }
        fn set_fullscreen(&self, fullscreen: bool) {
            *self.shadow_fullscreen.borrow_mut() = fullscreen;
        }
        fn set_fullscreen_init(&self, fullscreen: bool) {
            self.window_handle.set_fullscreen(fullscreen);
            *self.shadow_fullscreen.borrow_mut() = fullscreen;
        }
    }
}

#[cfg(not(target_os = "windows"))]
mod graphical_env {
    use super::graphical::WindowHandle;
    use orbital_decay_app::Env;
    pub struct GraphicalEnv {
        window_handle: WindowHandle,
    }
    impl GraphicalEnv {
        pub fn new(window_handle: WindowHandle) -> Self {
            Self { window_handle }
        }
    }
    impl Env for GraphicalEnv {
        fn fullscreen(&self) -> bool {
            self.window_handle.fullscreen()
        }
        fn fullscreen_requires_restart(&self) -> bool {
            false
        }
        fn fullscreen_supported(&self) -> bool {
            super::FULLSCREEN_SUPPORTED
        }
        fn set_fullscreen(&self, fullscreen: bool) {
            self.window_handle.set_fullscreen(fullscreen)
        }
        fn set_fullscreen_init(&self, fullscreen: bool) {
            self.window_handle.set_fullscreen(fullscreen)
        }
    }
}

use graphical::*;
use graphical_env::*;

struct Args {
    native_common: NativeCommon,
    fullscreen: Option<Fullscreen>,
}

impl Args {
    fn parser() -> impl meap::Parser<Item = Self> {
        meap::let_map! {
            let {
                native_common = NativeCommon::parser();
                fullscreen = flag('f').name("fullscreen").desc("start in fullscreen");
            } in {{
                let fullscreen = if fullscreen {
                    Some(Fullscreen)
                } else {
                    None
                };
                Self { native_common, fullscreen }
            }}
        }
    }
}

fn main() {
    use meap::Parser;
    env_logger::init();
    let Args {
        native_common:
            NativeCommon {
                rng_seed,
                file_storage,
                controls,
                save_file,
                audio_player,
                game_config,
            },
        fullscreen,
    } = Args::parser().with_help_default().parse_env_or_exit();
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
    let env = GraphicalEnv::new(context.window_handle());
    let app = app(
        game_config,
        Frontend::Graphical,
        controls,
        file_storage,
        save_file,
        audio_player,
        rng_seed,
        None,
        fullscreen,
        Box::new(env),
    );
    context.run_app(app);
}
