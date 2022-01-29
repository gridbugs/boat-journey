use chargrid_web::{Context, Size};
use general_audio_static::{backend::WebAudioPlayer, StaticAudioPlayer};
use general_storage_static::{backend::LocalStorage, StaticStorage};
use orbital_decay_app_wip::{app, AppArgs, AppStorage, InitialRngSeed};
use wasm_bindgen::prelude::*;

const SAVE_KEY: &str = "save";
const CONFIG_KEY: &str = "config";

#[wasm_bindgen(start)]
pub fn run() -> Result<(), JsValue> {
    wasm_logger::init(wasm_logger::Config::new(log::Level::Info));
    console_error_panic_hook::set_once();
    let audio_player = Some(StaticAudioPlayer::new(WebAudioPlayer::new_with_mime(
        "video/ogg",
    )));
    let storage = StaticStorage::new(LocalStorage::new());
    let context = Context::new(Size::new(80, 60), "content");
    let args = AppArgs {
        storage: AppStorage {
            handle: storage,
            save_game_key: SAVE_KEY.to_string(),
            config_key: CONFIG_KEY.to_string(),
        },
        initial_rng_seed: InitialRngSeed::Random,
        audio_player,
        omniscient: false,
        new_game: false,
    };
    context.run(app(args));
    Ok(())
}
