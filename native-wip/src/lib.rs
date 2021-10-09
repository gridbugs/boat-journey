use general_storage_static::{
    backend::{FileStorage, IfDirectoryMissing},
    StaticStorage,
};
pub use meap;
use orbital_decay_app_wip::{RngSeed, SaveGameStorage};

const DEFAULT_SAVE_FILE: &str = "save";
const DEFAULT_NEXT_TO_EXE_SAVE_DIR: &str = "save";

pub struct NativeCommon {
    pub save_game_storage: SaveGameStorage,
    pub rng_seed: RngSeed,
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
            } in {{
                let rng_seed = rng_seed.map(RngSeed::U64).unwrap_or(RngSeed::Random);
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
                let save_game_storage = SaveGameStorage {
                    handle: file_storage,
                    key: save_file,
                };
                Self {
                    rng_seed,
                    save_game_storage,
                    omniscient,
                }
            }}
        }
    }
}
