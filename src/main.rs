mod game;
mod random;
mod vec2i;

use std::path::Path;
use std::process::ExitCode;
use std::thread;
use std::time::Duration;

use raylib::prelude::*;

use game::camera_controller_3d::CameraController3d;
use game::escape_shake::EscapeShake;
use game::flashlight::Flashlight;
use game::map::{self, Map};
use game::musics::Musics;
use game::player::Player;
use game::renderer::{HudInfo, Renderer};
use game::sounds::Sounds;
use game::textures::Textures;
use game::yura::Yura;
use random::Random;

const ASSETS_DIR: &str = "_assets";

struct Args {
    window_size: Option<(i32, i32)>,
    max_fps: u32,
}

const USAGE: &str = "Usage: yura_escape [--window-size <WIDTH> <HEIGHT>] [--max-fps <FPS>]

Options:
  --window-size <WIDTH> <HEIGHT>  set window size [default: half the monitor size]
  --max-fps <FPS>                 max ingame fps [default: 240]
  -h, --help                      print this help";

fn parse_args() -> Result<Option<Args>, String> {
    let mut args = Args {
        window_size: None,
        max_fps: 240,
    };

    let mut iter = std::env::args().skip(1);
    while let Some(arg) = iter.next() {
        let mut value = |name: &str| -> Result<String, String> {
            iter.next()
                .ok_or_else(|| format!("{arg}: missing value for {name}"))
        };

        match arg.as_str() {
            "--window-size" => {
                let width = value("WIDTH")?;
                let height = value("HEIGHT")?;
                let width = width
                    .parse()
                    .map_err(|_| format!("invalid width: {width}"))?;
                let height = height
                    .parse()
                    .map_err(|_| format!("invalid height: {height}"))?;
                args.window_size = Some((width, height));
            }
            "--max-fps" => {
                let fps = value("FPS")?;
                args.max_fps = fps.parse().map_err(|_| format!("invalid fps: {fps}"))?;
            }
            "-h" | "--help" => return Ok(None),
            other => return Err(format!("unknown argument: {other}")),
        }
    }

    Ok(Some(args))
}

fn main() -> ExitCode {
    let args = match parse_args() {
        Ok(Some(args)) => args,
        Ok(None) => {
            println!("{USAGE}");
            return ExitCode::SUCCESS;
        }
        Err(err) => {
            eprintln!("{err}\n\n{USAGE}");
            return ExitCode::FAILURE;
        }
    };

    if Path::new(ASSETS_DIR).is_dir()
        && let Err(err) = std::env::set_current_dir(ASSETS_DIR)
    {
        eprintln!("failed to enter {ASSETS_DIR}: {err}");
        return ExitCode::FAILURE;
    }

    match run(&args) {
        Ok(()) => ExitCode::SUCCESS,
        Err(err) => {
            eprintln!("error: {err}");
            ExitCode::FAILURE
        }
    }
}

fn run(args: &Args) -> Result<(), Box<dyn std::error::Error>> {
    let (initial_width, initial_height) = args.window_size.unwrap_or((800, 600));
    let (mut rl, thread) = raylib::init()
        .size(initial_width, initial_height)
        .title("Yura Escape")
        .msaa_4x()
        .build();

    let monitor = get_current_monitor();
    let monitor_width = get_monitor_width(monitor);
    let monitor_height = get_monitor_height(monitor);

    let (window_width, window_height) = args
        .window_size
        .unwrap_or((monitor_width / 2, monitor_height / 2));
    rl.set_window_size(window_width, window_height);

    let center_x = monitor_width / 2 - window_width / 2;
    let center_y = monitor_height / 2 - window_height / 2;

    rl.set_exit_key(None);
    let mut escape_shake = EscapeShake::new();

    rl.disable_cursor();
    rl.set_target_fps(args.max_fps);

    let mut camera_controller = CameraController3d::new();
    let mut player = Player::default();
    let mut yura = Yura::default();

    let audio = RaylibAudio::init_audio_device()?;

    let textures = Textures::load_all(&mut rl, &thread)?;
    let sounds = Sounds::load_all(&audio)?;
    let musics = Musics::load_all(&audio)?;

    let mut renderer = Renderer::new(&mut rl, &thread)?;

    let mut flashlight = Flashlight::new(&camera_controller.current_camera);

    let mut last_footstep = 0.0f32;
    let mut footstep_rng = Random::new();

    let mut time = 0.0f32;
    let mut distance_from_yura = 100.0f32;

    let Some(mut map) = Map::load(&mut rl, &thread, "1", &textures, renderer.lighting_shader())?
    else {
        map::play_game_completed(&audio);
        return Ok(());
    };

    player.position = map.player_spawn;
    player.stamina = player.max_stamina;
    yura.position = map.yura_spawn;
    yura.speed = 1.0;

    audio.set_master_volume(0.15);

    sounds.end.set_volume(2.0);

    musics.near.play_stream();
    musics.near.set_volume(0.0);

    musics.ambient.play_stream();
    musics.ambient.set_volume(1.0);

    sounds.game_start.play();

    while !rl.window_should_close() {
        let frame_time = rl.get_frame_time();

        if rl.is_key_pressed(KeyboardKey::KEY_F) {
            flashlight.enabled = !flashlight.enabled;
        }

        time += frame_time;

        player.update(&rl, &map, &mut camera_controller);
        camera_controller.update(&rl, &mut player, distance_from_yura);
        flashlight.update(frame_time, &camera_controller.current_camera);

        let target_window_x = center_x as f32
            + ((camera_controller.shake.x * 32.0) - camera_controller.given_offset.x
                + camera_controller.bobbing.x)
                * 256.0;
        let target_window_y = center_y as f32
            + ((camera_controller.shake.y * 32.0) - camera_controller.given_offset.y
                + camera_controller.bobbing.y)
                * 256.0;

        let escape_offset = escape_shake.offset();

        rl.set_window_position(
            (target_window_x + escape_offset.x).round() as i32,
            (target_window_y + escape_offset.y).round() as i32,
        );

        if player.move_direction.length() >= 1.0
            && time - last_footstep >= 0.15
            && camera_controller.bobbing.y <= -0.02
            && player.landed
        {
            let step = &sounds.footsteps[footstep_rng.get_i32(0, 3) as usize];

            if !step.is_playing() {
                step.play();
                last_footstep = time;
                camera_controller.shake_amount += 0.25 * player.move_speed;
            }
        }

        if escape_shake.update(&rl, frame_time) {
            break;
        }

        renderer.draw_frame(
            &mut rl,
            &thread,
            &flashlight,
            &camera_controller.current_camera,
            &mut map.models,
            &textures,
            yura.position,
            &HudInfo {
                time,
                stamina: player.stamina,
            },
            &mut escape_shake,
        );

        yura.goal = player.position;
        yura.update(frame_time, &map);

        let to_yura = player.position - yura.position;
        distance_from_yura = to_yura.dot(to_yura);

        yura.speed += frame_time / 240.0;

        let near_volume = (40.0 - distance_from_yura).clamp(0.0, 40.0) / 10.0;
        musics.near.set_volume(near_volume);

        musics.ambient.update_stream();
        musics.near.update_stream();

        if distance_from_yura <= 1.0 {
            sounds.end.play();
            thread::sleep(Duration::from_millis(400));
            break;
        }
    }

    Ok(())
}
