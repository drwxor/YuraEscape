use raylib::prelude::*;

use crate::random::Random;

const PRESS_IMPULSE: f32 = 1.0;
const DECAY_RATE: f32 = 3.0;
const SHAKE_SMOOTH: f32 = 4.0;
const EXIT_THRESHOLD: f32 = 3.0;
const PIXELS_PER_PRESSURE: f32 = 16.0;

pub struct EscapeShake {
    pressure: f32,
    offset: Vector2,
    rng: Random,
}

fn map_range(val: f32, in_min: f32, in_max: f32, out_min: f32, out_max: f32) -> f32 {
    out_min + (val - in_min) * (out_max - out_min) / (in_max - in_min)
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
        self.offset = self.offset.lerp(
            Vector2::new(
                self.rng.get_f32(-amplitude, amplitude),
                self.rng.get_f32(-amplitude, amplitude),
            ) * map_range(self.pressure, 0.0, 3.0, 0.0, 128.0),
            1.0 - (-SHAKE_SMOOTH * frame_time).exp(),
        );

        false
    }

    pub fn draw(&mut self, d: &mut RaylibDrawHandle) {
        if self.pressure <= 0.1 {
            return;
        }

        let width = d.get_render_width();
        let height = d.get_render_height();

        for _ in 1..self.rng.get_u8(5, 20) {
            d.draw_text(
                "THERE IS NO ESCAPE",
                self.rng.get_i32(0, width),
                self.rng.get_i32(0, height),
                self.rng.get_i32(20, 60),
                Color::new(
                    255,
                    0,
                    0,
                    map_range(
                        self.pressure * self.rng.get_f32(0.25, 1.0),
                        0.0,
                        3.0,
                        0.0,
                        255.0,
                    ) as u8,
                ),
            );
        }
    }

    pub fn offset(&self) -> Vector2 {
        self.offset
    }
}
