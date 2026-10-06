use raylib::core::error::Error;
use raylib::ffi;
use raylib::prelude::*;

use crate::game::config;
use crate::game::escape_shake::EscapeShake;
use crate::game::flashlight::Flashlight;
use crate::game::lighting_shader::LightingShaderManager;
use crate::game::textures::Textures;

const SHADOW_RES: i32 = 1024;

pub struct HudInfo {
    pub time: f32,
    pub stamina: f32,
}

pub struct Renderer {
    shader_manager: LightingShaderManager,
}

impl Renderer {
    pub fn new(rl: &mut RaylibHandle, thread: &RaylibThread) -> Result<Self, Error> {
        Ok(Self {
            shader_manager: LightingShaderManager::new(rl, thread, SHADOW_RES as u32)?,
        })
    }

    pub fn lighting_shader(&self) -> &Shader {
        &self.shader_manager.lighting_shader
    }

    #[allow(clippy::too_many_arguments)]
    pub fn draw_frame(
        &mut self,
        rl: &mut RaylibHandle,
        thread: &RaylibThread,
        flashlight: &Flashlight,
        view_camera: &Camera3D,
        models: &mut [Model],
        textures: &Textures,
        yura_position: Vector3,
        hud: &HudInfo,
        escape_shake: &mut EscapeShake,
    ) {
        let LightingShaderManager {
            depth_shader,
            lighting_shader,
            shadow_target,
            locs,
        } = &mut self.shader_manager;
        let locs = *locs;

        let cam_pos = view_camera.position;
        let light_camera = flashlight.light_camera;
        let light_pos = light_camera.position;
        let spot_dir = (light_camera.target - light_pos).normalized();

        let light_view =
            Matrix::look_at(light_camera.position, light_camera.target, light_camera.up);
        let light_proj = Matrix::perspective(light_camera.fovy.to_radians(), 1.0, 0.01, 100.0);
        let light_vp = light_view * light_proj;

        {
            depth_shader.set_shader_value(locs.depth_light_pos, light_pos);
            let depth_raw = *depth_shader.as_ref();

            let mut texture_mode = rl.begin_texture_mode(thread, shadow_target);
            texture_mode.clear_background(Color::WHITE);

            let mut mode_3d = texture_mode.begin_mode3D(light_camera);
            unsafe { ffi::rlSetMatrixProjection(light_proj.into()) };

            let mut shader_mode = mode_3d.begin_shader_mode(depth_shader);
            draw_scene_geometry(&mut shader_mode, models, depth_raw);
        }

        lighting_shader.set_shader_value(locs.view_pos, cam_pos);
        lighting_shader.set_shader_value(locs.light_pos, light_pos);
        lighting_shader.set_shader_value(locs.spot_dir, spot_dir);
        lighting_shader.set_shader_value(locs.inner, flashlight.inner_cos);
        lighting_shader.set_shader_value(locs.outer, flashlight.outer_cos);

        let color = flashlight.light_color;
        lighting_shader.set_shader_value(locs.light_color, [color.x, color.y, color.z, 1.0]);
        lighting_shader.set_shader_value(locs.shadow_res, SHADOW_RES);
        lighting_shader.set_shader_value_matrix(locs.light_vp, light_vp);
        lighting_shader.set_shader_value_texture(locs.shadow_map, shadow_target.texture());

        let lighting_raw = *lighting_shader.as_ref();

        let mut d = rl.begin_drawing(thread);
        d.clear_background(Color::BLACK);

        {
            let mut mode_3d = d.begin_mode3D(*view_camera);
            {
                let mut shader_mode = mode_3d.begin_shader_mode(lighting_shader);
                draw_scene_geometry(&mut shader_mode, models, lighting_raw);
            }

            mode_3d.draw_billboard(
                view_camera,
                &textures.yura,
                yura_position,
                1.0,
                Color::WHITE,
            );
        }

        draw_gui(&mut d, hud);

        escape_shake.draw(&mut d);
    }
}

fn draw_scene_geometry(d: &mut impl RaylibDraw3D, models: &mut [Model], shader: ffi::Shader) {
    for model in models {
        let material = &mut model.materials_mut()[0];
        let original_shader = material.as_ref().shader;

        material.as_mut().shader = shader;
        d.draw_model(&*model, Vector3::zero(), 1.0, Color::WHITE);
        model.materials_mut()[0].as_mut().shader = original_shader;
    }
}

fn draw_gui(d: &mut RaylibDrawHandle, hud: &HudInfo) {
    let screen_width = d.get_screen_width();
    let screen_height = d.get_screen_height();

    if config::DEBUG {
        d.draw_fps(10, 10);
    }

    let title = "ESCAPE YURA";
    let title_width = d.measure_text(title, 10);
    d.draw_text(title, (screen_width - title_width) / 2, 12, 10, Color::RED);

    let survived = format!("YOU SURVIVED: {}", hud.time.floor());
    let survived_width = d.measure_text(&survived, 10);
    d.draw_text(
        &survived,
        (screen_width - survived_width) / 2,
        2,
        10,
        Color::RED,
    );

    let stamina = format!("STAMINA: {}", hud.stamina.floor());
    let stamina_width = d.measure_text(&stamina, 14);
    d.draw_text(
        &stamina,
        (screen_width - stamina_width) / 2,
        (screen_height as f32 * 0.9) as i32,
        14,
        Color::SKYBLUE,
    );
}
