use std::collections::HashSet;
use std::path::Path;
use std::thread;
use std::time::Duration;

use raylib::ffi;
use raylib::prelude::*;

use crate::game::textures::Textures;

fn cube_faces(half: f32) -> [[Vector3; 4]; 6] {
    let p0 = Vector3::new(-half, -half, half);
    let p1 = Vector3::new(half, -half, half);
    let p2 = Vector3::new(half, half, half);
    let p3 = Vector3::new(-half, half, half);
    let p4 = Vector3::new(-half, -half, -half);
    let p5 = Vector3::new(half, -half, -half);
    let p6 = Vector3::new(half, half, -half);
    let p7 = Vector3::new(-half, half, -half);

    [
        [p0, p1, p2, p3],
        [p5, p4, p7, p6],
        [p3, p2, p6, p7],
        [p4, p5, p1, p0],
        [p1, p5, p6, p2],
        [p4, p0, p3, p7],
    ]
}

unsafe fn raylib_buffer<T: Copy>(data: &[T]) -> *mut T {
    let bytes = std::mem::size_of_val(data);
    let ptr = unsafe { ffi::MemAlloc(bytes as u32) } as *mut T;
    unsafe { std::ptr::copy_nonoverlapping(data.as_ptr(), ptr, data.len()) };
    ptr
}

fn cell_key(pos: Vector3, cube_size: f32) -> (i32, i32, i32) {
    (
        (pos.x / cube_size).round() as i32,
        (pos.y / cube_size).round() as i32,
        (pos.z / cube_size).round() as i32,
    )
}

pub fn merge_cubes_into_mesh(positions: &[Vector3], cube_size: f32) -> Result<WeakMesh, String> {
    const QUAD_UV: [[f32; 2]; 4] = [[0.0, 1.0], [1.0, 1.0], [1.0, 0.0], [0.0, 0.0]];

    let occupied: HashSet<(i32, i32, i32)> = positions
        .iter()
        .map(|&pos| cell_key(pos, cube_size))
        .collect();

    let faces = cube_faces(cube_size * 0.5).map(|quad| {
        let normal = (quad[1] - quad[0]).cross(quad[2] - quad[0]).normalized();
        (quad, normal)
    });

    let mut verts_list: Vec<f32> = Vec::new();
    let mut norms_list: Vec<f32> = Vec::new();
    let mut uvs_list: Vec<f32> = Vec::new();
    let mut inds_list: Vec<u16> = Vec::new();

    for &pos in positions {
        for (quad, normal) in &faces {
            if occupied.contains(&cell_key(pos + *normal * cube_size, cube_size)) {
                continue;
            }

            let base_index = verts_list.len() / 3;
            if base_index + 4 > u16::MAX as usize {
                return Err(
                    "Too many vertices for single Mesh. Split your mesh into chunks!".to_owned(),
                );
            }
            let base_index = base_index as u16;

            for (corner, uv) in quad.iter().zip(QUAD_UV) {
                verts_list.extend([corner.x + pos.x, corner.y + pos.y, corner.z + pos.z]);
                norms_list.extend([normal.x, normal.y, normal.z]);
                uvs_list.extend(uv);
            }

            inds_list.extend([
                base_index,
                base_index + 1,
                base_index + 2,
                base_index,
                base_index + 2,
                base_index + 3,
            ]);
        }
    }

    unsafe {
        let mut mesh: ffi::Mesh = std::mem::zeroed();
        mesh.vertexCount = (verts_list.len() / 3) as i32;
        mesh.triangleCount = (inds_list.len() / 3) as i32;
        mesh.vertices = raylib_buffer(&verts_list);
        mesh.normals = raylib_buffer(&norms_list);
        mesh.texcoords = raylib_buffer(&uvs_list);
        mesh.indices = raylib_buffer(&inds_list);

        ffi::UploadMesh(&mut mesh, false);

        Ok(WeakMesh::from_raw(mesh))
    }
}

pub struct Map {
    pub width: i32,
    pub height: i32,

    pub tiles: Vec<u8>,
    #[allow(dead_code)]
    pub exits: Vec<u8>,
    #[allow(dead_code)]
    pub injector_positions: Vec<Vector3>,

    pub models: Vec<Model>,

    pub player_spawn: Vector3,
    pub yura_spawn: Vector3,
}

impl Map {
    pub fn load(
        rl: &mut RaylibHandle,
        thread: &RaylibThread,
        name: &str,
        textures: &Textures,
        lighting_shader: &Shader,
    ) -> Result<Option<Self>, Box<dyn std::error::Error>> {
        let map_path = format!("maps/{name}.png");

        if !Path::new(&map_path).exists() {
            return Ok(None);
        }

        println!("{map_path}");
        let image = Image::load_image(&map_path)?;

        let width = image.width();
        let height = image.height();
        let cell_count = (width * height) as usize;

        let mut tiles = vec![0u8; cell_count];
        let mut exits = vec![0u8; cell_count];
        let mut injector_positions = Vec::new();
        let mut player_spawn = Vector3::zero();
        let mut yura_spawn = Vector3::zero();

        let mut wall_positions = Vec::new();
        let mut floor_positions = Vec::new();
        let mut ceiling_positions = Vec::new();

        let map_colors = image.get_image_data();

        for y in 0..height {
            for x in 0..width {
                let index = (y * width + x) as usize;
                let pixel = map_colors[index];
                let (fx, fy) = (x as f32, y as f32);

                match (pixel.r, pixel.g, pixel.b) {
                    (0, 0, 0) => {
                        tiles[index] = 1;
                        wall_positions.push(Vector3::new(fx, 0.0, fy));
                    }
                    (0, 0, 255) => exits[index] = 1,
                    (255, 0, 255) => injector_positions.push(Vector3::new(fx, 0.0, fy)),
                    (0, 255, 0) => player_spawn = Vector3::new(fx, 0.0, fy),
                    (255, 0, 0) => yura_spawn = Vector3::new(fx, 0.0, fy),
                    _ => {}
                }

                if tiles[index] == 0 {
                    floor_positions.push(Vector3::new(fx, -1.0, fy));
                    ceiling_positions.push(Vector3::new(fx, 1.0, fy));
                }
            }
        }

        textures
            .floor
            .set_texture_wrap(thread, TextureWrap::TEXTURE_WRAP_REPEAT);
        textures
            .ceiling
            .set_texture_wrap(thread, TextureWrap::TEXTURE_WRAP_REPEAT);

        let mut models = Vec::with_capacity(3);
        for (positions, texture) in [
            (&wall_positions, &textures.wall),
            (&floor_positions, &textures.floor),
            (&ceiling_positions, &textures.ceiling),
        ] {
            let mesh = merge_cubes_into_mesh(positions, 1.0)?;
            let mut model = rl.load_model_from_mesh(thread, mesh)?;

            let material = &mut model.materials_mut()[0];
            material.set_material_texture(MaterialMapIndex::MATERIAL_MAP_ALBEDO, texture);
            material.as_mut().shader = *lighting_shader.as_ref();

            models.push(model);
        }

        Ok(Some(Self {
            width,
            height,
            tiles,
            exits,
            injector_positions,
            models,
            player_spawn,
            yura_spawn,
        }))
    }

    pub fn tile(&self, x: i32, z: i32) -> u8 {
        self.tiles[(z * self.width + x) as usize]
    }
}

pub fn play_game_completed(audio: &RaylibAudio) {
    println!("You completed the game for now!11!!!");

    let final_sound = audio.new_sound("audio/final.mp3").ok();
    let end_sfx = audio.new_sound("audio/end.mp3").ok();

    if let Some(sound) = &final_sound {
        sound.play();
    }
    thread::sleep(Duration::from_millis(1000));

    if let Some(sound) = &end_sfx {
        sound.play();
    }
    thread::sleep(Duration::from_millis(100));
}
