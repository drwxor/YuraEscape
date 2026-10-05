use crate::vec2i::Vec2i;

struct Direction {
    dx: i32,
    dy: i32,
    cost: f32,
}

fn in_bounds(x: i32, y: i32, w: i32, h: i32) -> bool {
    x >= 0 && y >= 0 && x < w && y < h
}

fn heuristic(x: i32, y: i32, gx: i32, gy: i32, diagonal_allowed: bool) -> f32 {
    let dx = (gx - x).abs();
    let dy = (gy - y).abs();
    if diagonal_allowed {
        ((dx * dx + dy * dy) as f32).sqrt()
    } else {
        (dx + dy) as f32
    }
}

fn reconstruct_path(
    came_from: &[Option<Vec2i>],
    width: i32,
    start: Vec2i,
    goal: Vec2i,
) -> Vec<Vec2i> {
    let mut path = vec![goal];
    let mut current = goal;

    while current != start {
        match came_from[(current.y * width + current.x) as usize] {
            Some(prev) => {
                current = prev;
                path.push(current);
            }
            None => break,
        }
    }

    path.reverse();
    path
}

pub fn find_path(
    map: &[u8],
    width: i32,
    height: i32,
    start: Vec2i,
    goal: Vec2i,
    allow_diagonals: bool,
) -> Vec<Vec2i> {
    if !in_bounds(start.x, start.y, width, height) || !in_bounds(goal.x, goal.y, width, height) {
        return Vec::new();
    }

    let index = |x: i32, y: i32| (y * width + x) as usize;

    if map[index(start.x, start.y)] != 0 || map[index(goal.x, goal.y)] != 0 {
        return Vec::new();
    }

    if start == goal {
        return vec![start];
    }

    let mut dirs = vec![
        Direction {
            dx: 1,
            dy: 0,
            cost: 1.0,
        },
        Direction {
            dx: -1,
            dy: 0,
            cost: 1.0,
        },
        Direction {
            dx: 0,
            dy: 1,
            cost: 1.0,
        },
        Direction {
            dx: 0,
            dy: -1,
            cost: 1.0,
        },
    ];

    if allow_diagonals {
        let diag = 2.0f32.sqrt();
        dirs.extend([
            Direction {
                dx: 1,
                dy: 1,
                cost: diag,
            },
            Direction {
                dx: 1,
                dy: -1,
                cost: diag,
            },
            Direction {
                dx: -1,
                dy: 1,
                cost: diag,
            },
            Direction {
                dx: -1,
                dy: -1,
                cost: diag,
            },
        ]);
    }

    let cell_count = (width * height) as usize;
    let mut g_score = vec![1e9f32; cell_count];
    let mut f_score = vec![1e9f32; cell_count];
    let mut in_open_set = vec![false; cell_count];
    let mut came_from: Vec<Option<Vec2i>> = vec![None; cell_count];

    let mut open_list = vec![start];

    g_score[index(start.x, start.y)] = 0.0;
    f_score[index(start.x, start.y)] = heuristic(start.x, start.y, goal.x, goal.y, allow_diagonals);
    in_open_set[index(start.x, start.y)] = true;

    while !open_list.is_empty() {
        let mut best_idx = 0;
        let mut best_f = f_score[index(open_list[0].x, open_list[0].y)];

        for (i, cell) in open_list.iter().enumerate().skip(1) {
            let f = f_score[index(cell.x, cell.y)];
            if f < best_f {
                best_f = f;
                best_idx = i;
            }
        }

        let current = open_list.remove(best_idx);
        in_open_set[index(current.x, current.y)] = false;

        if current == goal {
            return reconstruct_path(&came_from, width, start, goal);
        }

        for dir in &dirs {
            let nx = current.x + dir.dx;
            let ny = current.y + dir.dy;

            if !in_bounds(nx, ny, width, height) || map[index(nx, ny)] != 0 {
                continue;
            }

            if allow_diagonals
                && dir.dx != 0
                && dir.dy != 0
                && (map[index(current.x + dir.dx, current.y)] != 0
                    || map[index(current.x, current.y + dir.dy)] != 0)
            {
                continue;
            }

            let tentative_g = g_score[index(current.x, current.y)] + dir.cost;
            let n = index(nx, ny);

            if tentative_g < g_score[n] {
                came_from[n] = Some(current);
                g_score[n] = tentative_g;
                f_score[n] = tentative_g + heuristic(nx, ny, goal.x, goal.y, allow_diagonals);

                if !in_open_set[n] {
                    in_open_set[n] = true;
                    open_list.push(Vec2i::new(nx, ny));
                }
            }
        }
    }

    Vec::new()
}
