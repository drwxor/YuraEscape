use std::fmt;

use raylib::prelude::Vector2;

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct Vec2i {
    pub x: i32,
    pub y: i32,
}

impl Vec2i {
    pub const fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }

    #[allow(dead_code)]
    pub fn to_vector2(self) -> Vector2 {
        Vector2::new(self.x as f32, self.y as f32)
    }
}

impl fmt::Display for Vec2i {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "({},{})", self.x, self.y)
    }
}
