use raylib::prelude::*;

use crate::random::Random;

const PRESS_IMPULSE: f32 = 1.0;
const DECAY_RATE: f32 = 3.0;
const EXIT_THRESHOLD: f32 = 3.0;
const PIXELS_PER_PRESSURE: f32 = 16.0;

pub struct EscapeShake {
    pressure: f32,
    offset: Vector2,
    rng: Random,
}

impl EscapeShake {
    pub fn new() -> Self {
        Self {
            pressure: 0.0,
            offset: Vector2::zero(),
            rng: Random::new(),
        }
    }

    pub fn update(&mut self, rl: &RaylibHandle, frame_time: f32) -> bool {
        if rl.is_key_pressed(KeyboardKey::KEY_ESCAPE) {
            self.pressure += PRESS_IMPULSE;
        }

        if self.pressure >= EXIT_THRESHOLD {
            return true;
        }

        self.pressure *= (-DECAY_RATE * frame_time).exp();

        let amplitude = self.pressure * PIXELS_PER_PRESSURE;
        self.offset = Vector2::new(
            self.rng.get_f32(-amplitude, amplitude),
            self.rng.get_f32(-amplitude, amplitude),
        );

        false
    }

    pub fn offset(&self) -> Vector2 {
        self.offset
    }
}
