use rayon::{prelude::*, result};
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

pub fn shore_adjacent_water_coords(terrain_map: &Vec<Vec<f64>>, water_lvl: f64) -> Vec<(usize, usize)> {
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
) -> Vec<(i32, usize, usize)> {
    let mut water_body_groups = Vec::new();
    let mut visited = vec![vec![false; width]; height];
    let mut group_id: i32 = 0;

    
    for y in 0..height {
        for x in 0..width {
            if terrain_map[y][x] > water_lvl || visited[y][x] {
                continue;
            }

            let mut queue = VecDeque::from([(x, y)]);
            visited[y][x] = true;

            while let Some((curr_x, curr_y)) = queue.pop_front() {
                water_body_groups.push((group_id, curr_x, curr_y));

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

            group_id += 1;
        }
    }

    water_body_groups
}

pub fn water_body_group_size(mut water_body_groups: Vec<(i32, usize, usize)>) -> Vec<(i32, usize)> {
    
    let mut result_sizes: Vec<(i32, usize)> = Vec::new();
    if water_body_groups.len() == 0 {
        return result_sizes;
    }

    water_body_groups.sort_by_key(|tup| tup.0);
    let mut curr_group = water_body_groups[0].0;
    let mut curr_group_size = 0;
    
    for curr_cell in water_body_groups.iter() {
        if curr_cell.0 == curr_group {
            curr_group_size += 1;
            continue;
        }

        result_sizes.push((curr_group, curr_group_size));
        curr_group_size = 1;
        curr_group = curr_cell.0;
    }

    result_sizes.push((curr_group, curr_group_size));
    result_sizes
}