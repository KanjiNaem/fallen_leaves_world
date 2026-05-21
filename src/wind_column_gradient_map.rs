// gen norm_perlin map 
// set lambda (0.2-0.5 ish) 
// gen dist_ocean_map
// gen phy map: Φ = dist_ocean + lambda \* norm_perlin; --> scalar potential map
// gen upwind_neighbor_map with acyclic edge case decission; --> one parent per cell acyclic map
// gen rainfall map from this

use crate::perlin_greyscale;
use std::collections::VecDeque;
use rayon::prelude::*;

pub fn gen_wind_column_gradient_map(width: usize, height: usize, terrain_map: &Vec<Vec<f64>>, map_start_period: usize, water_lvl: f64) -> Vec<Vec<Option<(usize, usize)>>> {

    let mut norm_perl = perlin_greyscale::gen_single_layer_perlin_greyscale(width, height, map_start_period * 4);
    let (min_val, max_val) = norm_perl
        .par_iter()
        .flatten()
        .fold(
            || (f64::INFINITY, f64::NEG_INFINITY),
            |(min, max), &curr_val| (min.min(curr_val), max.max(curr_val)),
        )
        .reduce(
            || (f64::INFINITY, f64::NEG_INFINITY),
            |(acc_min, acc_max), (next_min, next_max)| (acc_min.min(next_min), acc_max.max(next_max)),
        );

    let range = (max_val - min_val).max(f64::EPSILON);
    
    norm_perl.par_iter_mut().for_each(|row| {
        for curr_val in row {
            *curr_val = (*curr_val - min_val) / range;
        }
    });

    let lambda = 0.2;
    let ocean_dist_map = gen_dist_ocean_map(terrain_map, width, height, water_lvl);
    let phi_current_map =
        gen_phi_current_map(terrain_map, width, height, water_lvl, &ocean_dist_map, &norm_perl, lambda);
    let upwind_neighbor_map =
        gen_upwind_map(width, height, &phi_current_map, &ocean_dist_map, terrain_map, water_lvl);

    return upwind_neighbor_map;
}

const CARDINAL_DELTAS: [(isize, isize); 4] = [(1, 0), (-1, 0), (0, 1), (0, -1)];

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

fn shore_adjacent_water_coords(terrain_map: &Vec<Vec<f64>>, water_lvl: f64) -> Vec<(usize, usize)> {
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

fn gen_dist_ocean_map(terrain_map: &Vec<Vec<f64>>, width: usize, height: usize, water_lvl: f64) -> Vec<Vec<f64>> {
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


// order: lower Φ first; then closer to ocean; then fixed cell index
type FlowRank = (f64, f64, usize);

fn get_flow_rank(phi: f64, dist_ocean: f64, x: usize, y: usize, width: usize) -> FlowRank {
    (phi, dist_ocean, x + width * y)
}

/// Φ = dist_ocean + lambda * norm_perlin (Φ = 0 for ocean tiles)
fn gen_phi_current_map(
    terrain_map: &Vec<Vec<f64>>,
    width: usize,
    height: usize,
    water_lvl: f64,
    ocean_dist_map: &Vec<Vec<f64>>,
    norm_perlin: &Vec<Vec<f64>>,
    lambda: f64,
) -> Vec<Vec<f64>> {
    let mut phi = vec![vec![0.0; width]; height];

    for y in 0..height {
        for x in 0..width {
            if terrain_map[y][x] > water_lvl {
                phi[y][x] = ocean_dist_map[y][x] + lambda * norm_perlin[y][x];
            }
        }
    }

    phi
}

fn pick_upwind_land(
    x: usize,
    y: usize,
    width: usize,
    height: usize,
    phi: &Vec<Vec<f64>>,
    ocean_dist_map: &Vec<Vec<f64>>,
    terrain_map: &Vec<Vec<f64>>,
    water_lvl: f64,
) -> Option<(usize, usize)> {
    // lower rank wins --> wind along neg gradient 
    let flow_rank = get_flow_rank(phi[y][x], ocean_dist_map[y][x], x, y, width);
    let mut best_choice: Option<(FlowRank, (usize, usize))> = None;

    for &(dx, dy) in &CARDINAL_DELTAS {
        let nx = x as isize + dx;
        let ny = y as isize + dy;
        if nx < 0 || ny < 0 {
            continue;
        }
        let (nx, ny) = (nx as usize, ny as usize);
        if nx >= width || ny >= height || terrain_map[ny][nx] <= water_lvl {
            continue;
        }

        let neighbor_rank = get_flow_rank(phi[ny][nx], ocean_dist_map[ny][nx], nx, ny, width);
        if neighbor_rank < flow_rank && best_choice.as_ref().map_or(true, |(curr_best_neighbor, _)| neighbor_rank < *curr_best_neighbor) {
            best_choice = Some((neighbor_rank, (nx, ny)));
        }
    }

    best_choice.map(|(_, coord)| coord)
}

fn gen_upwind_map(
    map_width: usize,
    map_height: usize,
    phi_map: &Vec<Vec<f64>>,
    ocean_dist_map: &Vec<Vec<f64>>,
    terrain_map: &Vec<Vec<f64>>,
    water_lvl: f64,
) -> Vec<Vec<Option<(usize, usize)>>> {
    (0..map_height)
        .into_par_iter()
        .map(|y| {
            (0..map_width)
                .map(|x| {
                    let is_ocean = terrain_map[y][x] <= water_lvl;
                    if is_ocean {
                        return None;
                    }

                    pick_upwind_land(
                        x,
                        y,
                        map_width,
                        map_height,
                        phi_map,
                        ocean_dist_map,
                        terrain_map,
                        water_lvl,
                    )
                })
                .collect()
        })
        .collect()
}