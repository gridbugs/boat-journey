use gridbugs::{
    audio::{AudioPlayer, NativeAudioError, NativeAudioPlayer},
    storage::{FileStorage, IfDirectoryMissing, Storage},
};
pub use meap;
use boat_journey_app::{AppAudioPlayer, AppStorage, InitialRngSeed};

const DEFAULT_SAVE_FILE: &str = "save";
const DEFAULT_NEXT_TO_EXE_STORAGE_DIR: &str = "save";
const DEFAULT_CONFIG_FILE: &str = "config.json";
const DEFAULT_CONTROLS_FILE: &str = "controls.json";

pub struct NativeCommon {
    pub storage: AppStorage,
    pub initial_rng_seed: InitialRngSeed,
    pub audio_player: AppAudioPlayer,
    pub omniscient: bool,
    pub new_game: bool,
}
impl NativeCommon {
    pub fn parser() -> impl meap::Parser<Item = Self> {
        meap::let_map! {
            let {
                rng_seed = opt_opt::<u64, _>("INT", 'r').name("rng-seed").desc("rng seed to use for first new game");
                save_file = opt_opt("PATH", 's').name("save-file").desc("save file")
                    .with_default(DEFAULT_SAVE_FILE.to_string());
                config_file = opt_opt("PATH", 'c').name("config-file").desc("config file")
                    .with_default(DEFAULT_CONFIG_FILE.to_string());
                controls_file = opt_opt("PATH", "controls-file").desc("controls file")
                    .with_default(DEFAULT_CONTROLS_FILE.to_string());
                storage_dir = opt_opt("PATH", 'd').name("storage-dir")
                    .desc("directory that will contain state")
                    .with_default(DEFAULT_NEXT_TO_EXE_STORAGE_DIR.to_string());
                delete_save = flag("delete-save").desc("delete save game file");
                delete_config = flag("delete-config").desc("delete config file");
                new_game = flag("new-game").desc("start a new game, skipping the menu");
                omniscient = flag("omniscient").desc("enable omniscience");
                mute = flag('m').name("mute").desc("mute audio");
            } in {{
                let initial_rng_seed = rng_seed.map(InitialRngSeed::U64).unwrap_or(InitialRngSeed::Random);
                let mut file_storage = Storage::new(
                    FileStorage::next_to_exe(storage_dir, IfDirectoryMissing::Create)
                    .expect("failed to open directory"),
                );
                if delete_save {
                    let result = file_storage.remove(&save_file);
                    if result.is_err() {
                        log::warn!("couldn't find save file to delete");
                    }
                }
                if delete_config {
                    let result = file_storage.remove(&config_file);
                    if result.is_err() {
                        log::warn!("couldn't find config file to delete");
                    }
                }
                let storage = AppStorage {
                    handle: file_storage,
                    save_game_key: save_file,
                    config_key: config_file,
                    controls_key: controls_file,
                };
                let audio_player = if mute {
                    None
                } else {
                    match NativeAudioPlayer::try_new_default_device() {
                        Ok(audio_player) => Some(AudioPlayer::new(audio_player)),
                        Err(NativeAudioError::FailedToCreateOutputStream) => {
                            log::warn!("no output audio device - continuing without audio");
                            None
                        }
                    }
                };
                Self {
                    initial_rng_seed,
                    storage,
                    audio_player,
                    omniscient,
                    new_game,
                }
            }}
        }
    }
}
