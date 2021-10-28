use chargrid_web::{Context, Size};
use general_storage_static::{backend::LocalStorage, StaticStorage};
use orbital_decay_app_wip::{app, AppArgs, InitialRngSeed, SaveGameStorage};
use wasm_bindgen::prelude::*;

const SAVE_KEY: &str = "save";

#[wasm_bindgen(start)]
pub fn run() -> Result<(), JsValue> {
    wasm_logger::init(wasm_logger::Config::new(log::Level::Info));
    console_error_panic_hook::set_once();
    let storage = StaticStorage::new(LocalStorage::new());
    let context = Context::new(Size::new(80, 60), "content");
    let args = AppArgs {
        save_game_storage: SaveGameStorage {
            handle: storage,
            key: SAVE_KEY.to_string(),
        },
        initial_rng_seed: InitialRngSeed::Random,
        omniscient: false,
    };
    context.run(app(args));
    Ok(())
}
