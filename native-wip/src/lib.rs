use general_storage_static::{
    backend::{FileStorage, IfDirectoryMissing},
    StaticStorage,
};
use orbital_decay_app_wip::SaveGameStorage;

pub struct NativeCommon {
    pub save_game_storage: SaveGameStorage,
}
impl NativeCommon {
    pub fn new() -> Self {
        let file_storage = StaticStorage::new(
            FileStorage::next_to_exe("save", IfDirectoryMissing::Create)
                .expect("failed to open directory"),
        );
        let save_game_storage = SaveGameStorage {
            handle: file_storage,
            key: "save".to_string(),
        };
        Self { save_game_storage }
    }
}
