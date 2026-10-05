use raylib::prelude::*;

pub struct Flashlight {
    pub inner_cos: f32,
    pub outer_cos: f32,
    pub light_color: Vector3,
    pub light_camera: Camera3D,
    pub enabled: bool,
}

impl Flashlight {
    pub fn new(view_camera: &Camera3D) -> Self {
        Self {
            inner_cos: 30.0f32.to_radians().cos(),
            outer_cos: 50.0f32.to_radians().cos(),
            light_color: Vector3::new(1.0, 0.95, 0.8),
            light_camera: Camera3D::perspective(
                view_camera.position,
                view_camera.target,
                Vector3::new(0.0, 1.0, 0.0),
                50.0,
            ),
            enabled: false,
        }
    }

    pub fn update(&mut self, delta_time: f32, view_camera: &Camera3D) {
        let look_vec = view_camera.target - view_camera.position;

        self.light_camera.position = view_camera.position;

        let target_offset_pos = if self.enabled {
            self.light_camera.position + look_vec
        } else {
            let drop_vector = look_vec * -2.0 - Vector3::new(0.0, 1.0, 0.0);
            self.light_camera.position + drop_vector
        };

        self.light_camera.target = self
            .light_camera
            .target
            .lerp(target_offset_pos, 1.0 - (-8.0 * delta_time).exp());
    }
}
