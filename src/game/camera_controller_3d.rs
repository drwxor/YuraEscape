use raylib::prelude::*;

use crate::game::constants::HALF_RAD;
use crate::game::player::Player;
use crate::random::Random;

fn rotate_by_axis_angle(v: Vector3, axis: Vector3, angle: f32) -> Vector3 {
    let k = axis.normalized();
    let (sin, cos) = angle.sin_cos();
    v * cos + k.cross(v) * sin + k * (k.dot(v) * (1.0 - cos))
}

pub struct CameraController3d {
    pitch: f32,
    yaw: f32,

    walk_distance: f32,

    offset_rotation: Vector3,

    rng: Random,

    pub current_camera: Camera3D,

    pub min_pitch: f32,
    pub max_pitch: f32,

    pub sensitivity: f32,

    pub fov: f32,
    pub fov_add_target: f32,
    pub fov_add: f32,

    pub zoom: f32,
    pub zoom_target: f32,

    pub lean: f32,
    pub lean_target: f32,

    pub shake_amount: f32,

    pub bobbing: Vector3,

    pub given_offset: Vector3,
    pub given_offset_target: Vector3,

    pub offset: Vector3,

    pub shake: Vector3,

    pub z: f32,
}

impl CameraController3d {
    pub fn new() -> Self {
        let fov = 90.0;

        Self {
            pitch: 0.0,
            yaw: 0.0,
            walk_distance: 0.0,
            offset_rotation: Vector3::zero(),
            rng: Random::new(),
            current_camera: Camera3D::perspective(
                Vector3::new(0.0, 2.0, 10.0),
                Vector3::new(0.0, 2.0, 0.0),
                Vector3::new(0.0, 1.0, 0.0),
                fov,
            ),
            min_pitch: -89.0f32.to_radians(),
            max_pitch: 89.0f32.to_radians(),
            sensitivity: 0.005,
            fov,
            fov_add_target: 0.0,
            fov_add: 0.0,
            zoom: 0.0,
            zoom_target: 0.0,
            lean: 0.0,
            lean_target: 0.0,
            shake_amount: 0.0,
            bobbing: Vector3::zero(),
            given_offset: Vector3::zero(),
            given_offset_target: Vector3::zero(),
            offset: Vector3::zero(),
            shake: Vector3::zero(),
            z: 0.0,
        }
    }

    pub fn update(&mut self, rl: &RaylibHandle, player: &mut Player, distance_from_yura: f32) {
        let frame_time = rl.get_frame_time();
        let mouse_delta = rl.get_mouse_delta() * self.sensitivity;

        let current_move_speed = player.move_direction.length() * player.move_speed;

        self.offset = Vector3::zero();
        self.offset_rotation = Vector3::zero();

        self.pitch -= mouse_delta.y;
        self.yaw -= mouse_delta.x;

        if player.landed {
            self.walk_distance += frame_time * current_move_speed * 4.0;
        }

        let dt_decay_fast = 1.0 - (-16.0 * frame_time).exp();
        let dt_decay_med = 1.0 - (-8.0 * frame_time).exp();
        let dt_decay_normal = 1.0 - (-4.0 * frame_time).exp();

        self.shake_amount = lerp(self.shake_amount, 0.0, dt_decay_med);

        let target_bobbing = Vector3::new(
            self.walk_distance.sin() - self.walk_distance.cos(),
            (self.walk_distance * 2.0).sin(),
            0.0,
        );
        self.bobbing = self.bobbing.lerp(
            target_bobbing * (current_move_speed * 0.05),
            dt_decay_normal,
        );

        let target_shake = Vector3::new(
            self.rng.get_f32(-1.0, 1.0),
            self.rng.get_f32(-1.0, 1.0),
            0.0,
        );
        self.shake = self
            .shake
            .lerp(target_shake * (self.shake_amount * 0.05), dt_decay_normal);

        let mouse_roll = if frame_time > 0.0 {
            (mouse_delta.x / self.sensitivity) / (frame_time * 60.0)
        } else {
            0.0
        };
        let target_z =
            (player.raw_move_direction.x * 0.15 * current_move_speed) + (mouse_roll * 0.02);
        self.z = lerp(self.z, target_z * 0.25, dt_decay_normal);

        self.offset_rotation += self.bobbing * (frame_time * 6.0);
        self.offset_rotation.z += self.z + self.lean;

        self.offset.y += self.bobbing.y + (player.height - 2.0);
        self.offset += self.shake;
        self.offset += player.right_vector * self.lean;

        self.lean_target = if rl.is_key_down(KeyboardKey::KEY_E) {
            1.0
        } else if rl.is_key_down(KeyboardKey::KEY_Q) {
            -1.0
        } else {
            0.0
        };
        self.lean = lerp(self.lean, self.lean_target / 4.0, dt_decay_normal);

        let random_rot_shake = Vector3::new(
            self.rng.get_f32(-1.0, 1.0),
            self.rng.get_f32(-1.0, 1.0),
            self.rng.get_f32(-8.0, 8.0),
        );
        self.offset_rotation += random_rot_shake * (0.0025 * self.shake_amount);

        self.pitch += self.offset_rotation.y;
        self.yaw += self.offset_rotation.x;

        self.given_offset_target = self
            .given_offset_target
            .lerp(Vector3::zero(), dt_decay_normal);
        self.given_offset = self
            .given_offset
            .lerp(self.given_offset_target, dt_decay_fast);

        self.offset += self.given_offset;

        self.pitch = self.pitch.clamp(self.min_pitch, self.max_pitch);

        let (pitch, yaw) = (self.pitch, self.yaw);

        player.look_vector = Vector3::new(
            pitch.cos() * yaw.sin(),
            pitch.sin(),
            pitch.cos() * yaw.cos(),
        );
        player.right_vector = Vector3::new((yaw - HALF_RAD).sin(), 0.0, (yaw - HALF_RAD).cos());
        player.up_vector = Vector3::new(
            (pitch + HALF_RAD).cos() * yaw.sin(),
            (pitch + HALF_RAD).sin(),
            (pitch + HALF_RAD).cos() * yaw.cos(),
        );
        player.move_vector = Vector3::new(yaw.sin(), 0.0, yaw.cos());

        if distance_from_yura < 40.0 {
            self.shake_amount += (40.0 - distance_from_yura).sqrt() * frame_time * 3.0;
        }

        if rl.is_mouse_button_pressed(MouseButton::MOUSE_BUTTON_LEFT)
            || rl.is_mouse_button_pressed(MouseButton::MOUSE_BUTTON_RIGHT)
        {
            self.shake_amount += 0.5;
        }

        self.zoom_target = if rl.is_key_down(KeyboardKey::KEY_C) {
            45.0
        } else {
            0.0
        };

        self.fov_add = lerp(
            self.fov_add,
            self.fov_add_target + (1.0 - (current_move_speed - 1.0)) * 9.0,
            1.0 - (-6.0 * frame_time).exp(),
        );

        self.fov = 90.0 - self.fov_add - self.zoom;
        self.zoom = lerp(self.zoom, self.zoom_target, dt_decay_normal);

        self.current_camera.fovy = self.fov;
        self.current_camera.position = player.position + self.offset;
        self.current_camera.target = self.current_camera.position + player.look_vector;
        self.current_camera.up =
            rotate_by_axis_angle(Vector3::up(), player.look_vector, self.offset_rotation.z);
    }
}
