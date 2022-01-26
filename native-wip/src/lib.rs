use general_audio_static::{
    backend::{Error as NativeAudioError, NativeAudioPlayer},
    StaticAudioPlayer,
};
use general_storage_static::{
    backend::{FileStorage, IfDirectoryMissing},
    StaticStorage,
};
pub use meap;
use orbital_decay_app_wip::{AppAudioPlayer, AppStorage, InitialRngSeed};

const DEFAULT_SAVE_FILE: &str = "save";
const DEFAULT_NEXT_TO_EXE_SAVE_DIR: &str = "save";

pub struct NativeCommon {
    pub storage: AppStorage,
    pub initial_rng_seed: InitialRngSeed,
    pub audio_player: AppAudioPlayer,
    pub omniscient: bool,
}
impl NativeCommon {
    pub fn parser() -> impl meap::Parser<Item = Self> {
        meap::let_map! {
            let {
                rng_seed = opt_opt::<u64, _>("INT", 'r').name("rng-seed").desc("rng seed to use for first new game");
                save_file = opt_opt("PATH", 's').name("save-file").desc("save file")
                    .with_default(DEFAULT_SAVE_FILE.to_string());
                save_dir = opt_opt("PATH", 'd').name("save-dir").desc("save dir")
                    .with_default(DEFAULT_NEXT_TO_EXE_SAVE_DIR.to_string());
                delete_save = flag("delete-save").desc("delete save game file");
                omniscient = flag("omniscient").desc("enable omniscience");
                mute = flag('m').name("mute").desc("mute audio");
            } in {{
                let initial_rng_seed = rng_seed.map(InitialRngSeed::U64).unwrap_or(InitialRngSeed::Random);
                let mut file_storage = StaticStorage::new(
                    FileStorage::next_to_exe(save_dir, IfDirectoryMissing::Create)
                    .expect("failed to open directory"),
                );
                if delete_save {
                    let result = file_storage.remove(&save_file);
                    if result.is_err() {
                        log::warn!("couldn't find save file to delete");
                    }
                }
                let storage = AppStorage {
                    handle: file_storage,
                    save_game_key: save_file,
                };
                let audio_player = if mute {
                    None
                } else {
                    match NativeAudioPlayer::try_new_default_device() {
                        Ok(audio_player) => Some(StaticAudioPlayer::new(audio_player)),
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
                }
            }}
        }
    }
}
