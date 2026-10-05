use raylib::prelude::*;

use crate::game::camera_controller_3d::CameraController3d;
use crate::game::config;
use crate::game::map::Map;

pub struct Player {
    pub position: Vector3,

    pub look_vector: Vector3,
    pub move_vector: Vector3,
    pub right_vector: Vector3,
    pub up_vector: Vector3,

    pub velocity: Vector3,

    pub move_direction: Vector3,
    pub raw_move_direction: Vector3,

    pub move_speed: f32,
    pub default_walk_speed: f32,
    pub walk_speed: f32,

    pub injector_active: bool,
    pub hurtjector_active: bool,
    #[allow(dead_code)]
    pub hurtjector_value: f32,

    pub sprint_multiplier: f32,
    pub crouch_multiplier: f32,

    pub acceleration: f32,
    pub sprint_drain_rate: f32,
    pub stamina_regen_rate: f32,

    pub height: f32,
    pub radius: f32,

    pub stamina: f32,
    pub max_stamina: f32,

    pub landed: bool,
}

impl Default for Player {
    fn default() -> Self {
        Self {
            position: Vector3::zero(),
            look_vector: Vector3::zero(),
            move_vector: Vector3::zero(),
            right_vector: Vector3::zero(),
            up_vector: Vector3::zero(),
            velocity: Vector3::zero(),
            move_direction: Vector3::zero(),
            raw_move_direction: Vector3::zero(),
            move_speed: 0.0,
            default_walk_speed: 1.0,
            walk_speed: 1.0,
            injector_active: false,
            hurtjector_active: false,
            hurtjector_value: 1.0,
            sprint_multiplier: 2.5,
            crouch_multiplier: 0.5,
            acceleration: 12.0,
            sprint_drain_rate: 4.0,
            stamina_regen_rate: 2.0,
            height: 2.0,
            radius: 0.1,
            stamina: 100.0,
            max_stamina: 100.0,
            landed: false,
        }
    }
}

impl Player {
    fn resolve_collision(&mut self, map: &Map) {
        let radius_sq = self.radius * self.radius;

        for _ in 0..4 {
            let mut any_push = false;

            let min_x = (self.position.x - self.radius - 0.5).floor() as i32;
            let max_x = (self.position.x + self.radius + 0.5).floor() as i32;
            let min_z = (self.position.z - self.radius - 0.5).floor() as i32;
            let max_z = (self.position.z + self.radius + 0.5).floor() as i32;

            for tx in min_x.max(0)..=max_x.min(map.width - 1) {
                for tz in min_z.max(0)..=max_z.min(map.height - 1) {
                    if map.tile(tx, tz) == 0 {
                        continue;
                    }

                    let tile_min_y = -0.5;
                    let tile_max_y = 0.5;
                    if self.position.y + self.height < tile_min_y || self.position.y > tile_max_y {
                        continue;
                    }

                    let closest_x = self.position.x.clamp(tx as f32 - 0.5, tx as f32 + 0.5);
                    let closest_z = self.position.z.clamp(tz as f32 - 0.5, tz as f32 + 0.5);

                    let dx = self.position.x - closest_x;
                    let dz = self.position.z - closest_z;
                    let dist_sq = dx * dx + dz * dz;

                    if dist_sq >= radius_sq {
                        continue;
                    }

                    let dist = dist_sq.sqrt();
                    let penetration = self.radius - dist;

                    let push_dir = if dist > 1e-6 {
                        Vector3::new(dx / dist, 0.0, dz / dist)
                    } else {
                        let ddx = self.position.x - tx as f32;
                        let ddz = self.position.z - tz as f32;
                        let len = (ddx * ddx + ddz * ddz).sqrt();

                        if len > 1e-6 {
                            Vector3::new(ddx / len, 0.0, ddz / len)
                        } else {
                            Vector3::new(1.0, 0.0, 0.0)
                        }
                    };

                    self.position.x += push_dir.x * penetration;
                    self.position.z += push_dir.z * penetration;
                    any_push = true;
                }
            }

            if !any_push {
                break;
            }
        }
    }

    pub fn update(
        &mut self,
        rl: &RaylibHandle,
        map: &Map,
        camera_controller: &mut CameraController3d,
    ) {
        let frame_time = rl.get_frame_time();
        let mut input_vector = Vector3::zero();
        let target_land = self.position.y < 0.01;
        self.raw_move_direction = Vector3::zero();

        if rl.is_key_down(KeyboardKey::KEY_W) {
            self.raw_move_direction.z += 1.0;
            input_vector += self.move_vector;
        }
        if rl.is_key_down(KeyboardKey::KEY_S) {
            self.raw_move_direction.z -= 1.0;
            input_vector -= self.move_vector;
        }
        if rl.is_key_down(KeyboardKey::KEY_D) {
            self.raw_move_direction.x += 1.0;
            input_vector += self.right_vector;
        }
        if rl.is_key_down(KeyboardKey::KEY_A) {
            self.raw_move_direction.x -= 1.0;
            input_vector -= self.right_vector;
        }

        if rl.is_key_pressed(KeyboardKey::KEY_SPACE) && target_land {
            self.velocity.y += 3.0;
        }

        self.landed = target_land;
        self.move_direction = if input_vector.dot(input_vector) > 0.0 {
            input_vector.normalized()
        } else {
            Vector3::zero()
        };
        self.walk_speed = self.default_walk_speed;

        let want_run = rl.is_key_down(KeyboardKey::KEY_LEFT_SHIFT)
            && self.stamina > 0.0
            && self.move_direction.dot(self.move_direction) > 0.0;
        let want_crouch = rl.is_key_down(KeyboardKey::KEY_LEFT_CONTROL);

        if self.injector_active {
            self.walk_speed *= 1.5;
        }
        if self.hurtjector_active {
            self.walk_speed *= 0.12;
        }

        if want_run && !want_crouch {
            self.stamina = (self.stamina - self.sprint_drain_rate * frame_time).max(0.0);
            self.walk_speed *= self.sprint_multiplier;
        } else if self.stamina < self.max_stamina {
            self.stamina =
                (self.stamina + self.stamina_regen_rate * frame_time).min(self.max_stamina);
        }

        self.height = lerp(
            self.height,
            if want_crouch { 1.85 } else { 2.0 },
            1.0 - (-8.0 * frame_time).exp(),
        );
        if want_crouch {
            self.walk_speed *= self.crouch_multiplier;
        }

        let fall = frame_time * (config::GRAVITY / 16.0);
        let target_velocity =
            self.move_direction * (self.walk_speed * (1.0 + self.position.y * 4.0));

        if self.position.y > 0.0 {
            self.velocity.y -= fall;
        }

        let accel_t = 1.0 - (-self.acceleration * frame_time).exp();
        self.velocity.x = lerp(self.velocity.x, target_velocity.x, accel_t);
        self.velocity.z = lerp(self.velocity.z, target_velocity.z, accel_t);

        if self.position.y < 0.0 {
            self.position.y = 0.0;
        }

        self.position += self.velocity * frame_time;
        self.resolve_collision(map);

        self.move_speed = Vector3::new(self.velocity.x, 0.0, self.velocity.z).length();

        if self.position.y < 0.01 && !self.landed {
            self.landed = true;
            self.velocity.y = 0.0;
            camera_controller.given_offset_target.y -= 0.2;
        }
    }
}
