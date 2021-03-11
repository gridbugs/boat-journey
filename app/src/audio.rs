use general_audio_static::{AudioPlayer, StaticAudioPlayer, StaticHandle, StaticSound};
use maplit::hashmap;
use orbital_decay_game::SoundEffect;
use std::collections::HashMap;

pub type AppAudioPlayer = Option<StaticAudioPlayer>;
pub type AppSound = Option<StaticSound>;
pub type AppHandle = Option<StaticHandle>;

const GAMEPLAY0: &[u8] = include_bytes!("./audio/Terminant.ogg");
const GAMEPLAY1: &[u8] = include_bytes!("./audio/Disconnected.ogg");
const GAMEPLAY2: &[u8] = include_bytes!("./audio/Absolute+Terror.ogg");
const BOSS: &[u8] = include_bytes!("./audio/Panthalassa.ogg");
const END_TEXT: &[u8] = include_bytes!("./audio/Bush+Week.ogg");
const MENU: &[u8] = include_bytes!("./audio/10,000+People+Chanting,+-I'm+an+Individual-.ogg");
const EXPLOSION: &[u8] = include_bytes!("./audio/explosion.ogg");

#[derive(Clone, Copy, Hash, PartialEq, Eq, Debug)]
pub enum Audio {
    Gameplay0,
    Gameplay1,
    Gameplay2,
    Boss,
    EndText,
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
            Audio::Boss => audio_player.load_sound(BOSS),
            Audio::EndText => audio_player.load_sound(END_TEXT),
            Audio::Menu => audio_player.load_sound(MENU),
            Audio::Explosion => audio_player.load_sound(EXPLOSION),
            Audio::SoundEffect(SoundEffect::Shotgun) => audio_player.load_sound(EXPLOSION),
            Audio::SoundEffect(SoundEffect::Rifle) => audio_player.load_sound(EXPLOSION),
            Audio::SoundEffect(SoundEffect::Railgun) => audio_player.load_sound(EXPLOSION),
            Audio::SoundEffect(SoundEffect::GausCannon) => audio_player.load_sound(EXPLOSION),
            Audio::SoundEffect(SoundEffect::LifeStealer) => audio_player.load_sound(EXPLOSION),
            Audio::SoundEffect(SoundEffect::Oxidiser) => audio_player.load_sound(EXPLOSION),
            Audio::SoundEffect(SoundEffect::Chainsaw) => audio_player.load_sound(EXPLOSION),
            Audio::SoundEffect(SoundEffect::Punch) => audio_player.load_sound(EXPLOSION),
            Audio::SoundEffect(SoundEffect::DoorOpen) => audio_player.load_sound(EXPLOSION),
        ];
        Self { map }
    }
    pub fn get(&self, audio: Audio) -> &AppSound {
        self.map.get(&audio).unwrap()
    }
}
