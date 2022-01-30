use chargrid_web::{Context, Size};
use general_audio_static::{backend::WebAudioPlayer, StaticAudioPlayer};
use general_storage_static::{backend::LocalStorage, StaticStorage};
use orbital_decay_app_old::{app, Controls, EnvNull, Frontend, GameConfig, RngSeed};
use wasm_bindgen::prelude::*;

const SAVE_KEY: &str = "save";

#[wasm_bindgen(start)]
pub fn run() -> Result<(), JsValue> {
    wasm_logger::init(wasm_logger::Config::new(log::Level::Info));
    console_error_panic_hook::set_once();
    let audio_player = Some(StaticAudioPlayer::new(WebAudioPlayer::new_with_mime(
        "video/ogg",
    )));
    let storage = StaticStorage::new(LocalStorage::new());
    let context = Context::new(Size::new(80, 60), "content");
    let app = app(
        GameConfig {
            omniscient: None,
            demo: false,
            debug: false,
        },
        Frontend::Web,
        Controls::default(),
        storage,
        SAVE_KEY.to_string(),
        audio_player,
        RngSeed::Random,
        None,
        None,
        Box::new(EnvNull),
    );
    context.run_app(app);
    Ok(())
}
