use raylib::core::error::Error;
use raylib::prelude::*;

#[derive(Clone, Copy)]
pub struct ShaderLocations {
    pub ambient: i32,
    pub light_pos: i32,
    pub spot_dir: i32,
    pub inner: i32,
    pub outer: i32,
    pub light_color: i32,
    pub light_vp: i32,
    pub shadow_map: i32,
    pub shadow_res: i32,
    pub view_pos: i32,
    pub depth_light_pos: i32,
}

pub struct LightingShaderManager {
    pub depth_shader: Shader,
    pub lighting_shader: Shader,
    pub shadow_target: RenderTexture2D,
    pub locs: ShaderLocations,
}

impl LightingShaderManager {
    pub fn new(
        rl: &mut RaylibHandle,
        thread: &RaylibThread,
        shadow_res: u32,
    ) -> Result<Self, Error> {
        let depth_shader =
            rl.load_shader(thread, Some("shaders/depth.vs"), Some("shaders/depth.fs"))?;
        let mut lighting_shader = rl.load_shader(
            thread,
            Some("shaders/lighting.vs"),
            Some("shaders/lighting.fs"),
        )?;
        let shadow_target = rl.load_render_texture(thread, shadow_res, shadow_res)?;

        let locs = ShaderLocations {
            ambient: lighting_shader.get_shader_location("ambient"),
            light_pos: lighting_shader.get_shader_location("lightPos"),
            spot_dir: lighting_shader.get_shader_location("spotDir"),
            inner: lighting_shader.get_shader_location("spotInnerCos"),
            outer: lighting_shader.get_shader_location("spotOuterCos"),
            light_color: lighting_shader.get_shader_location("lightColor"),
            light_vp: lighting_shader.get_shader_location("lightVP"),
            shadow_map: lighting_shader.get_shader_location("shadowMap"),
            shadow_res: lighting_shader.get_shader_location("shadowMapResolution"),
            view_pos: lighting_shader.get_shader_location("viewPos"),
            depth_light_pos: depth_shader.get_shader_location("lightPos"),
        };

        lighting_shader.locs_mut()[ShaderLocationIndex::SHADER_LOC_VECTOR_VIEW as usize] =
            locs.view_pos;
        lighting_shader.set_shader_value(locs.ambient, [0.01f32, 0.01, 0.01, 1.0]);

        Ok(Self {
            depth_shader,
            lighting_shader,
            shadow_target,
            locs,
        })
    }
}
