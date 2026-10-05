use raylib::core::error::Error;
use raylib::prelude::*;

pub struct Sounds<'aud> {
    pub game_start: Sound<'aud>,
    pub end: Sound<'aud>,
    #[allow(dead_code)]
    pub random: Sound<'aud>,
    pub footsteps: Vec<Sound<'aud>>,
}

impl<'aud> Sounds<'aud> {
    pub fn load_all(audio: &'aud RaylibAudio) -> Result<Self, Error> {
        Ok(Self {
            game_start: audio.new_sound("audio/start.ogg")?,
            end: audio.new_sound("audio/end.mp3")?,
            random: audio.new_sound("audio/random.ogg")?,
            footsteps: vec![
                audio.new_sound("audio/footsteps/1.wav")?,
                audio.new_sound("audio/footsteps/2.wav")?,
                audio.new_sound("audio/footsteps/3.wav")?,
                audio.new_sound("audio/footsteps/4.wav")?,
            ],
        })
    }
}
