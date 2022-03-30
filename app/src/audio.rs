use general_audio_static::{StaticAudioPlayer, StaticHandle, StaticSound};

use maplit::hashmap;
use orbital_decay_game::SoundEffect;
use std::collections::HashMap;

pub type AppAudioPlayer = Option<StaticAudioPlayer>;
pub type AppHandle = Option<StaticHandle>;

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
    map: Option<HashMap<Audio, StaticSound>>,
}

impl AudioTable {
    pub fn new(audio_player: &AppAudioPlayer) -> Self {
        use audio_data::*;
        let map = audio_player.as_ref().map(|audio_player| {
        hashmap![
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
        ]
        });
        Self { map }
    }
    pub fn get(&self, audio: Audio) -> Option<&StaticSound> {
        self.map.as_ref().map(|map| map.get(&audio).unwrap())
    }
}

pub struct AudioState {
    audio_player: AppAudioPlayer,
    audio_table: AudioTable,
    music_handle: AppHandle,
    music_volume: f32,
    music_volume_multiplier: f32,
}

impl AudioState {
    pub fn new(audio_player: AppAudioPlayer) -> Self {
        let audio_table = AudioTable::new(&audio_player);
        Self {
            audio_player,
            audio_table,
            music_handle: None,
            music_volume: 1.,
            music_volume_multiplier: 1.,
        }
    }

    pub fn play_once(&self, audio: Audio, volume: f32) {
        log::info!("Playing audio {:?} at volume {:?}", audio, volume);
        if let Some(sound) = self.audio_table.get(audio) {
            if let Some(audio_player) = self.audio_player.as_ref() {
                let handle = audio_player.play(&sound);
                handle.set_volume(volume);
                handle.background();
            }
        }
    }

    pub fn loop_music(&mut self, audio: Audio, volume: f32) {
        log::info!("Looping audio {:?} at volume {:?}", audio, volume);
        if let Some(sound) = self.audio_table.get(audio) {
            if let Some(audio_player) = self.audio_player.as_ref() {
                let handle = audio_player.play_loop(&sound);
                handle.set_volume(volume * self.music_volume_multiplier);
                self.music_handle = Some(handle);
                self.music_volume = volume;
            }
        }
    }

    pub fn set_music_volume(&mut self, volume: f32) {
        self.music_volume = volume;
        if let Some(music_handle) = self.music_handle.as_mut() {
            music_handle.set_volume(volume * self.music_volume_multiplier);
        }
    }

    pub fn set_music_volume_multiplier(&mut self, music_volume_multiplier: f32) {
        self.music_volume_multiplier = music_volume_multiplier;
        if let Some(music_handle) = self.music_handle.as_mut() {
            music_handle.set_volume(self.music_volume * self.music_volume_multiplier);
        }
    }
}
