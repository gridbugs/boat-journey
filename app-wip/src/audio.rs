use general_audio_static::{
    AudioHandle, AudioPlayer, StaticAudioPlayer, StaticHandle, StaticSound,
};

use maplit::hashmap;
use orbital_decay_game::SoundEffect;
use std::collections::HashMap;

pub type AppAudioPlayer = Option<StaticAudioPlayer>;
pub type AppSound = Option<StaticSound>;
pub type AppHandle = Option<StaticHandle>;

const GAMEPLAY0: &[u8] = include_bytes!("./audio/Level 1.mp3");
const GAMEPLAY1: &[u8] = include_bytes!("./audio/Level 2.mp3");
const GAMEPLAY2: &[u8] = include_bytes!("./audio/Level 3.mp3");
const END_TEXT_HAPPY: &[u8] = include_bytes!("./audio/Orbital Decay.mp3");
const END_TEXT_SAD: &[u8] = include_bytes!("./audio/Sad Orbital Decay.mp3");
const MENU: &[u8] = include_bytes!("./audio/Menu.mp3");

const EXPLOSION: &[u8] = include_bytes!("./audio/Explosion.mp3");

const SHOTGUN: &[u8] = include_bytes!("./audio/Shotgun.mp3");
const RIFLE: &[u8] = include_bytes!("./audio/Rifle.mp3");
const RAILGUN: &[u8] = include_bytes!("./audio/Rail Gun.mp3");
const GAUS_CANNON: &[u8] = include_bytes!("./audio/Gaus Cannon.mp3");
const LIFE_STEALER: &[u8] = include_bytes!("./audio/Health Gun.mp3");
const OXIDISER: &[u8] = include_bytes!("./audio/Oxygen Gun.mp3");
const CHAINSAW: &[u8] = include_bytes!("./audio/Chainsaw.mp3");
const PUNCH: &[u8] = include_bytes!("./audio/Punch.mp3");
const DOOR_OPEN: &[u8] = include_bytes!("./audio/Science Fiction Door Opening.mp3");
const HEAL: &[u8] = include_bytes!("./audio/Heal.mp3");
const DIE: &[u8] = include_bytes!("./audio/Die.mp3");

#[derive(Clone, Copy, Hash, PartialEq, Eq, Debug)]
pub enum Audio {
    Gameplay0,
    Gameplay1,
    Gameplay2,
    EndTextHappy,
    EndTextSad,
    Menu,
    Explosion,
    SoundEffect(SoundEffect),
}

pub struct AudioTable {
    map: HashMap<Audio, AppSound>,
}

impl AudioTable {
    pub fn new(audio_player: &AppAudioPlayer) -> Self {
        let map = hashmap![
            Audio::Gameplay0 => audio_player.load_sound(GAMEPLAY0),
            Audio::Gameplay1 => audio_player.load_sound(GAMEPLAY1),
            Audio::Gameplay2=> audio_player.load_sound(GAMEPLAY2),
            Audio::EndTextHappy => audio_player.load_sound(END_TEXT_HAPPY),
            Audio::EndTextSad => audio_player.load_sound(END_TEXT_SAD),
            Audio::Menu => audio_player.load_sound(MENU),
            Audio::Explosion => audio_player.load_sound(EXPLOSION),
            Audio::SoundEffect(SoundEffect::Shotgun) => audio_player.load_sound(SHOTGUN),
            Audio::SoundEffect(SoundEffect::Rifle) => audio_player.load_sound(RIFLE),
            Audio::SoundEffect(SoundEffect::Railgun) => audio_player.load_sound(RAILGUN),
            Audio::SoundEffect(SoundEffect::GausCannon) => audio_player.load_sound(GAUS_CANNON),
            Audio::SoundEffect(SoundEffect::LifeStealer) => audio_player.load_sound(LIFE_STEALER),
            Audio::SoundEffect(SoundEffect::Oxidiser) => audio_player.load_sound(OXIDISER),
            Audio::SoundEffect(SoundEffect::Chainsaw) => audio_player.load_sound(CHAINSAW),
            Audio::SoundEffect(SoundEffect::Punch) => audio_player.load_sound(PUNCH),
            Audio::SoundEffect(SoundEffect::DoorOpen) => audio_player.load_sound(DOOR_OPEN),
            Audio::SoundEffect(SoundEffect::Heal) => audio_player.load_sound(HEAL),
            Audio::SoundEffect(SoundEffect::Die) => audio_player.load_sound(DIE),
        ];
        Self { map }
    }
    pub fn get(&self, audio: Audio) -> &AppSound {
        self.map.get(&audio).unwrap()
    }
}

pub struct AudioState {
    audio_player: AppAudioPlayer,
    audio_table: AudioTable,
    music_handle: Option<AppHandle>,
}

impl AudioState {
    pub fn new(audio_player: AppAudioPlayer) -> Self {
        let audio_table = AudioTable::new(&audio_player);
        Self {
            audio_player,
            audio_table,
            music_handle: None,
        }
    }

    pub fn play_once(&self, audio: Audio, volume: f32) {
        log::info!("Playing audio {:?} at volume {:?}", audio, volume);
        let sound = self.audio_table.get(audio);
        let handle = self.audio_player.play(&sound);
        handle.set_volume(volume);
        handle.background();
    }

    pub fn loop_music(&mut self, audio: Audio, volume: f32) {
        log::info!("Looping audio {:?} at volume {:?}", audio, volume);
        let sound = self.audio_table.get(audio);
        let handle = self.audio_player.play_loop(&sound);
        handle.set_volume(volume);
        self.music_handle = Some(handle);
    }

    pub fn set_music_volume(&mut self, volume: f32) {
        if let Some(music_handle) = self.music_handle.as_mut() {
            music_handle.set_volume(volume);
        }
    }
}
