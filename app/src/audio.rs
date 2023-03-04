use gridbugs::audio::{Audio as Sound, AudioHandle, AudioPlayer};

use maplit::hashmap;
use std::collections::HashMap;

pub type AppAudioPlayer = Option<AudioPlayer>;
pub type AppHandle = Option<AudioHandle>;

#[derive(Clone, Copy, Hash, PartialEq, Eq, Debug)]
pub enum Audio {}

pub struct AudioState {
    audio_player: AppAudioPlayer,
    music_handle: AppHandle,
    music_volume: f32,
    music_volume_multiplier: f32,
}

impl AudioState {
    pub fn new(audio_player: AppAudioPlayer) -> Self {
        Self {
            audio_player,
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
