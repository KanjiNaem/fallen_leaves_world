use fxhash::FxHashMap;
use rayon::prelude::*;
use std::collections::VecDeque;

const CARDINAL_DELTAS: [(isize, isize); 4] = [(1, 0), (-1, 0), (0, 1), (0, -1)];

pub fn gen_water_body_size(
    width: usize,
    height: usize,
    terrain_map: &Vec<Vec<f64>>,
    water_lvl: f64,
) -> Vec<Vec<f64>> {
    let mut water_body_size_map = vec![vec![0.0; width]; height];
    let mut visited = vec![vec![false; width]; height];

    for y in 0..height {
        for x in 0..width {
            if terrain_map[y][x] > water_lvl || visited[y][x] {
                continue;
            }

            let mut queue = VecDeque::from([(x, y)]);
            let mut component = Vec::new();
            visited[y][x] = true;

            while let Some((cx, cy)) = queue.pop_front() {
                component.push((cx, cy));
                for &(dx, dy) in &CARDINAL_DELTAS {
                    let nx = cx as isize + dx;
                    let ny = cy as isize + dy;
                    if nx < 0 || ny < 0 {
                        continue;
                    }
                    let (nx, ny) = (nx as usize, ny as usize);
                    if nx >= width
                        || ny >= height
                        || terrain_map[ny][nx] > water_lvl
                        || visited[ny][nx]
                    {
                        continue;
                    }
                    visited[ny][nx] = true;
                    queue.push_back((nx, ny));
                }
            }

            let size = component.len() as f64;
            for (cx, cy) in component {
                water_body_size_map[cy][cx] = size;
            }
        }
    }

    water_body_size_map
}

pub fn get_adj_height_diff_map(
    width: usize,
    height: usize,
    terrain_map: Vec<Vec<f64>>,
) -> Vec<Vec<(f64, f64)>> {
    // return vec![vec![(0.0, 0.0); 0]; 0];
    (0..height)
        .into_par_iter()
        .map(|y| {
            (0..width)
                .map(|x| {
                    let mut smallest_height = f64::INFINITY;
                    let mut largest_height = f64::NEG_INFINITY;
                    for &(cx, cy) in &CARDINAL_DELTAS {
                        if y as isize + cy < 0 || x as isize + cx < 0 {
                            continue;
                        }
                        if terrain_map[y][x] < smallest_height {
                            smallest_height = terrain_map[y][x];
                        }
                        if terrain_map[y][x] > largest_height {
                            largest_height = terrain_map[y][x];
                        }
                    }
                    (smallest_height, largest_height)
                })
                .collect()
        })
        .collect()
}

pub fn shore_adjacent_water_coords(
    terrain_map: &Vec<Vec<f64>>,
    water_lvl: f64,
) -> Vec<(usize, usize)> {
    let height = terrain_map.len();
    let width = terrain_map[0].len();

    terrain_map
        .par_iter()
        .enumerate()
        .flat_map_iter(|(y, row)| {
            (0..width).filter_map(move |x| {
                if row[x] > water_lvl {
                    return None;
                }
                borders_land(terrain_map, water_lvl, width, height, x, y).then_some((x, y))
            })
        })
        .collect()
}

fn borders_land(
    terrain_map: &Vec<Vec<f64>>,
    water_lvl: f64,
    width: usize,
    height: usize,
    x: usize,
    y: usize,
) -> bool {
    CARDINAL_DELTAS.iter().any(|&(dx, dy)| {
        let nx = x as isize + dx;
        let ny = y as isize + dy;
        if nx < 0 || ny < 0 {
            return false;
        }
        let (nx, ny) = (nx as usize, ny as usize);
        nx < width && ny < height && terrain_map[ny][nx] > water_lvl
    })
}

pub fn gen_dist_ocean_map(
    terrain_map: &Vec<Vec<f64>>,
    width: usize,
    height: usize,
    water_lvl: f64,
) -> Vec<Vec<f64>> {
    let mut dist_ocean_map = vec![vec![f64::INFINITY; width]; height];
    let mut queue = VecDeque::new();

    for &(x, y) in shore_adjacent_water_coords(terrain_map, water_lvl).iter() {
        dist_ocean_map[y][x] = 0.0;
        queue.push_back((x, y));
    }

    while let Some((x, y)) = queue.pop_front() {
        let next_dist = dist_ocean_map[y][x] + 1.0;
        for &(dx, dy) in &CARDINAL_DELTAS {
            let nx = x as isize + dx;
            let ny = y as isize + dy;
            if nx < 0 || ny < 0 {
                continue;
            }
            let (nx, ny) = (nx as usize, ny as usize);
            if nx >= width || ny >= height {
                continue;
            }
            if next_dist < dist_ocean_map[ny][nx] {
                dist_ocean_map[ny][nx] = next_dist;
                queue.push_back((nx, ny));
            }
        }
    }

    dist_ocean_map
}

pub fn group_water_bodies(
    width: usize,
    height: usize,
    terrain_map: &Vec<Vec<f64>>,
    water_lvl: f64,
) -> FxHashMap<i32, Vec<(usize, usize)>> {
    let mut water_body_groups: FxHashMap<i32, Vec<(usize, usize)>> = FxHashMap::default();
    let mut visited = vec![vec![false; width]; height];
    let mut group_id: i32 = 0;

    for y in 0..height {
        for x in 0..width {
            if terrain_map[y][x] > water_lvl || visited[y][x] {
                continue;
            }

            let mut queue = VecDeque::from([(x, y)]);
            let mut curr_group: Vec<(usize, usize)> = Vec::new();
            visited[y][x] = true;

            while let Some((curr_x, curr_y)) = queue.pop_front() {
                curr_group.push((curr_x, curr_y));
                for &(c_x, c_y) in &CARDINAL_DELTAS {
                    let next_x = curr_x as isize + c_x;
                    let next_y = curr_y as isize + c_y;
                    if next_x < 0 || next_y < 0 {
                        continue;
                    }

                    let (next_x, next_y) = (next_x as usize, next_y as usize);
                    if next_x >= width
                        || next_y >= height
                        || terrain_map[next_y][next_x] > water_lvl
                        || visited[next_y][next_x]
                    {
                        continue;
                    }
                    visited[next_y][next_x] = true;
                    queue.push_back((next_x, next_y));
                }
            }
            water_body_groups.insert(group_id, curr_group);
            group_id += 1;
        }
    }

    water_body_groups
}

pub fn water_body_group_size(
    water_body_groups: &FxHashMap<i32, Vec<(usize, usize)>>,
) -> FxHashMap<i32, usize> {
    water_body_groups
        .iter()
        .map(|(group_id, group_members)| (*group_id, group_members.len()))
        .collect()
}

pub fn get_shore_adj_for_body(
    width: usize,
    height: usize,
    body: &Vec<(usize, usize)>,
    terrain_map: &Vec<Vec<f64>>,
    water_lvl: f64,
) -> Vec<(usize, usize)> {
    body.iter()
        .copied()
        .filter(|&(x, y)| {
            terrain_map[y][x] <= water_lvl
                && borders_land(terrain_map, water_lvl, width, height, x, y)
        })
        .collect()
}

const MANHATTAN_DIST_INF: usize = usize::MAX / 4;

fn manhattan_dist_pass(grid: &mut Vec<Vec<usize>>) {
    let height = grid.len();
    let width = grid[0].len();

    for y in 0..height {
        for x in 1..width {
            grid[y][x] = grid[y][x].min(grid[y][x - 1].saturating_add(1));
        }
        for x in (0..width - 1).rev() {
            grid[y][x] = grid[y][x].min(grid[y][x + 1].saturating_add(1));
        }
    }

    for x in 0..width {
        for y in 1..height {
            grid[y][x] = grid[y][x].min(grid[y - 1][x].saturating_add(1));
        }
        for y in (0..height - 1).rev() {
            grid[y][x] = grid[y][x].min(grid[y + 1][x].saturating_add(1));
        }
    }
}

// L1 dist to nearest shore tile; cropped to shore bbox + infl_rad
pub fn gen_manhattan_dist_to_shore(
    width: usize,
    height: usize,
    shore: &[(usize, usize)],
    infl_rad: usize,
) -> (Vec<Vec<usize>>, usize, usize) {
    if shore.is_empty() {
        return (Vec::new(), 0, 0);
    }

    let mut min_x = width;
    let mut max_x = 0;
    let mut min_y = height;
    let mut max_y = 0;

    for &(x, y) in shore {
        min_x = min_x.min(x);
        max_x = max_x.max(x);
        min_y = min_y.min(y);
        max_y = max_y.max(y);
    }

    min_x = min_x.saturating_sub(infl_rad);
    max_x = (max_x + infl_rad).min(width - 1);
    min_y = min_y.saturating_sub(infl_rad);
    max_y = (max_y + infl_rad).min(height - 1);

    let sub_w = max_x - min_x + 1;
    let sub_h = max_y - min_y + 1;
    let mut grid = vec![vec![MANHATTAN_DIST_INF; sub_w]; sub_h];

    for &(x, y) in shore {
        grid[y - min_y][x - min_x] = 0;
    }

    manhattan_dist_pass(&mut grid);

    return (grid, min_x, min_y);
}
