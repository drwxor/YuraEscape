use raylib::prelude::*;

use crate::game::map::Map;
use crate::game::pathfinding;
use crate::vec2i::Vec2i;

const RECALC_INTERVAL: f32 = 0.25;

fn world_to_tile(world: f32) -> i32 {
    (world + 0.5).floor() as i32
}

pub struct Yura {
    pub position: Vector3,
    pub goal: Vector3,
    pub speed: f32,

    current_path: Vec<Vec2i>,
    path_index: usize,
    recalc_cooldown: f32,
}

impl Default for Yura {
    fn default() -> Self {
        Self {
            position: Vector3::zero(),
            goal: Vector3::zero(),
            speed: 3.0,
            current_path: Vec::new(),
            path_index: 0,
            recalc_cooldown: 0.0,
        }
    }
}

impl Yura {
    pub fn update(&mut self, dt: f32, map: &Map) {
        self.recalc_cooldown -= dt;

        let start = Vec2i::new(
            world_to_tile(self.position.x),
            world_to_tile(self.position.z),
        );
        let goal = Vec2i::new(world_to_tile(self.goal.x), world_to_tile(self.goal.z));

        let path_still_valid = self.recalc_cooldown > 0.0
            && self.path_index < self.current_path.len()
            && self.current_path.last() == Some(&goal);

        if !path_still_valid {
            self.recalc_cooldown = RECALC_INTERVAL;

            self.current_path =
                pathfinding::find_path(&map.tiles, map.width, map.height, start, goal, false);

            self.path_index = self
                .current_path
                .iter()
                .enumerate()
                .min_by(|(_, a), (_, b)| {
                    let da = (a.x as f32 - self.position.x).powi(2)
                        + (a.y as f32 - self.position.z).powi(2);
                    let db = (b.x as f32 - self.position.x).powi(2)
                        + (b.y as f32 - self.position.z).powi(2);
                    da.total_cmp(&db)
                })
                .map_or(0, |(i, _)| i);
        }

        if self.path_index + 1 >= self.current_path.len() {
            return;
        }

        let next_cell = self.current_path[self.path_index + 1];
        let target = Vector3::new(next_cell.x as f32, self.position.y, next_cell.y as f32);

        let dir = Vector3::new(target.x - self.position.x, 0.0, target.z - self.position.z);
        let dist = (dir.x * dir.x + dir.z * dir.z).sqrt();

        if dist <= 0.0001 {
            self.path_index += 1;
            return;
        }

        let step = self.speed * dt;

        if step >= dist || dist <= 0.05 {
            self.position.x = target.x;
            self.position.z = target.z;
            self.path_index += 1;
            return;
        }

        self.position.x += (dir.x / dist) * step;
        self.position.z += (dir.z / dist) * step;
    }
}
