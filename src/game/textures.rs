use raylib::core::error::Error;
use raylib::prelude::*;

pub struct Textures {
    pub floor: Texture2D,
    pub wall: Texture2D,
    pub ceiling: Texture2D,
    pub yura: Texture2D,
}

impl Textures {
    pub fn load_all(rl: &mut RaylibHandle, thread: &RaylibThread) -> Result<Self, Error> {
        Ok(Self {
            floor: rl.load_texture(thread, "textures/environment/floor.png")?,
            wall: rl.load_texture(thread, "textures/environment/wall.png")?,
            ceiling: rl.load_texture(thread, "textures/environment/ceiling.png")?,
            yura: rl.load_texture(thread, "textures/yura.png")?,
        })
    }
}
