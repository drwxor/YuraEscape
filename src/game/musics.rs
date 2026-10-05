use raylib::core::error::Error;
use raylib::prelude::*;

pub struct Musics<'aud> {
    pub ambient: Music<'aud>,
    pub near: Music<'aud>,
}

impl<'aud> Musics<'aud> {
    pub fn load_all(audio: &'aud RaylibAudio) -> Result<Self, Error> {
        Ok(Self {
            ambient: audio.new_music("audio/amb.ogg")?,
            near: audio.new_music("audio/near.ogg")?,
        })
    }
}
