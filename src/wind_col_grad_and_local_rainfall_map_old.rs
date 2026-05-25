// gen norm_perlin map
// set lambda (0.2-0.5 ish)
// gen dist_ocean_map
// gen phy map: Φ = dist_ocean + lambda \* norm_perlin; --> scalar potential map
// gen upwind_neighbor_map with acyclic edge case decission; --> one parent per cell acyclic map

use crate::{helpers, perlin_greyscale};
use rayon::prelude::*;

const BASE_RAINFALL_LEAK: f64 = 10.0;
const HEIGHT_DIFF_RAINFALL_COEF: f64 = 1.5;

fn rainfall_potential_from_body_size(body_size: f64, map_area: f64) -> f64 {
    let frac = body_size / map_area;
    if frac >= 0.10 {
        300000.0
    } else if frac >= 0.07 {
        250000.0
    } else if frac >= 0.05 {
        200000.0
    } else if frac >= 0.025 {
        100000.0
    } else if frac >= 0.01 {
        50000.0
    } else if frac >= 0.005 {
        10000.0
    } else {
        1000.0
    }
}

fn adjacent_water_rainfall_potential(
    x: usize,
    y: usize,
    width: usize,
    height: usize,
    terrain_map: &Vec<Vec<f64>>,
    water_body_size: &Vec<Vec<f64>>,
    water_lvl: f64,
    map_area: f64,
) -> f64 {
    CARDINAL_DELTAS
        .iter()
        .filter_map(|&(dx, dy)| {
            let nx = x as isize + dx;
            let ny = y as isize + dy;
            if nx < 0 || ny < 0 {
                return None;
            }

            let (nx, ny) = (nx as usize, ny as usize);
            if nx >= width || ny >= height || terrain_map[ny][nx] > water_lvl {
                return None;
            }

            Some(rainfall_potential_from_body_size(
                water_body_size[ny][nx],
                map_area,
            ))
        })
        .fold(0.0, f64::max)
}

fn land_rainfall_demand(
    terrain_map: &Vec<Vec<f64>>,
    water_lvl: f64,
    x: usize,
    y: usize,
    upwind: Option<(usize, usize)>,
) -> f64 {
    let height_diff = match upwind {
        Some((px, py)) => (terrain_map[y][x] - terrain_map[py][px]).max(0.0),
        None => (terrain_map[y][x] - water_lvl).max(0.0),
    };
    BASE_RAINFALL_LEAK + height_diff * HEIGHT_DIFF_RAINFALL_COEF
}

#[inline]
fn cell_index(x: usize, y: usize, width: usize) -> usize {
    y * width + x
}

fn precompute_coastal_source_potential(
    width: usize,
    height: usize,
    terrain_map: &Vec<Vec<f64>>,
    water_body_size: &Vec<Vec<f64>>,
    water_lvl: f64,
    map_area: f64,
) -> Vec<f64> {
    let mut coastal_source = vec![0.0; width * height];
    coastal_source
        .par_chunks_mut(width)
        .enumerate()
        .for_each(|(y, row)| {
            let terrain_row = &terrain_map[y];
            for x in 0..width {
                if terrain_row[x] <= water_lvl {
                    continue;
                }
                row[x] = adjacent_water_rainfall_potential(
                    x,
                    y,
                    width,
                    height,
                    terrain_map,
                    water_body_size,
                    water_lvl,
                    map_area,
                );
            }
        });
    coastal_source
}

pub fn gen_rainfall_from_flow_maps(
    width: usize,
    height: usize,
    terrain_map: &Vec<Vec<f64>>,
    water_lvl: f64,
    ocean_dist_map: &Vec<Vec<f64>>,
    phi_map: &Vec<Vec<f64>>,
    upwind_neighbor_map: &Vec<Vec<Option<(usize, usize)>>>,
) -> Vec<Vec<f64>> {
    let map_area = (width * height) as f64;
    let water_body_size = helpers::gen_water_body_size(width, height, terrain_map, water_lvl);
    let coastal_source = precompute_coastal_source_potential(
        width,
        height,
        terrain_map,
        &water_body_size,
        water_lvl,
        map_area,
    );

    let upwind = upwind_neighbor_map;
    let mut land_cells: Vec<(usize, usize, Option<(usize, usize)>)> = (0..height)
        .into_par_iter()
        .flat_map_iter(|y| {
            let upwind_row = &upwind[y];
            let terrain_row = &terrain_map[y];
            (0..width).filter_map(move |x| {
                if terrain_row[x] <= water_lvl {
                    return None;
                }
                Some((x, y, upwind_row[x]))
            })
        })
        .collect();

    land_cells.par_sort_unstable_by(|&(x_a, y_a, _), &(x_b, y_b, _)| {
        let rank_a = get_flow_rank(phi_map[y_a][x_a], ocean_dist_map[y_a][x_a], x_a, y_a, width);
        let rank_b = get_flow_rank(phi_map[y_b][x_b], ocean_dist_map[y_b][x_b], x_b, y_b, width);
        rank_a
            .partial_cmp(&rank_b)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    let mut rainfall_flat = vec![0.0; width * height];
    let mut stream_potential = vec![0.0; width * height];

    for (x, y, parent) in land_cells {
        let i = cell_index(x, y, width);
        let incoming_rainfall = match parent {
            Some((px, py)) => stream_potential[cell_index(px, py, width)],
            None => coastal_source[i],
        };

        if incoming_rainfall <= 0.0 {
            continue;
        }

        let rainfall_demand = land_rainfall_demand(terrain_map, water_lvl, x, y, parent);
        let rainfall_deposited = rainfall_demand.min(incoming_rainfall);
        rainfall_flat[i] = rainfall_deposited;
        stream_potential[i] = incoming_rainfall - rainfall_deposited;
    }

    (0..height)
        .into_par_iter()
        .map(|y| {
            let start = y * width;
            rainfall_flat[start..(start + width)].to_vec()
        })
        .collect()
}

pub fn gen_rainfall_map(
    width: usize,
    height: usize,
    terrain_map: &Vec<Vec<f64>>,
    water_lvl: f64,
    map_start_period: usize,
) -> Vec<Vec<f64>> {
    let (ocean_dist_map, phi_map) =
        gen_flow_rank_maps(width, height, terrain_map, map_start_period, water_lvl);
    let upwind_neighbor_map = gen_upwind_map(
        width,
        height,
        &phi_map,
        &ocean_dist_map,
        terrain_map,
        water_lvl,
    );
    gen_rainfall_from_flow_maps(
        width,
        height,
        terrain_map,
        water_lvl,
        &ocean_dist_map,
        &phi_map,
        &upwind_neighbor_map,
    )
}

pub fn gen_flow_rank_maps(
    width: usize,
    height: usize,
    terrain_map: &Vec<Vec<f64>>,
    map_start_period: usize,
    water_lvl: f64,
) -> (Vec<Vec<f64>>, Vec<Vec<f64>>) {
    let norm_perl = normalize_perlin_map(width, height, map_start_period * 4);
    let lambda = 0.2;
    let ocean_dist_map = helpers::gen_dist_ocean_map(terrain_map, width, height, water_lvl);
    let phi_current_map = gen_phi_current_map(
        terrain_map,
        width,
        height,
        water_lvl,
        &ocean_dist_map,
        &norm_perl,
        lambda,
    );
    (ocean_dist_map, phi_current_map)
}

pub fn gen_wind_column_gradient_map(
    width: usize,
    height: usize,
    terrain_map: &Vec<Vec<f64>>,
    map_start_period: usize,
    water_lvl: f64,
) -> Vec<Vec<Option<(usize, usize)>>> {
    let (ocean_dist_map, phi_current_map) =
        gen_flow_rank_maps(width, height, terrain_map, map_start_period, water_lvl);
    gen_upwind_map(
        width,
        height,
        &phi_current_map,
        &ocean_dist_map,
        terrain_map,
        water_lvl,
    )
}

fn normalize_perlin_map(width: usize, height: usize, period: usize) -> Vec<Vec<f64>> {
    let mut norm_perl = perlin_greyscale::gen_single_layer_perlin_greyscale(width, height, period);
    let (min_val, max_val) = norm_perl
        .par_iter()
        .flatten()
        .fold(
            || (f64::INFINITY, f64::NEG_INFINITY),
            |(min, max), &curr_val| (min.min(curr_val), max.max(curr_val)),
        )
        .reduce(
            || (f64::INFINITY, f64::NEG_INFINITY),
            |(acc_min, acc_max), (next_min, next_max)| {
                (acc_min.min(next_min), acc_max.max(next_max))
            },
        );

    let range = (max_val - min_val).max(f64::EPSILON);

    norm_perl.par_iter_mut().for_each(|row| {
        for curr_val in row {
            *curr_val = (*curr_val - min_val) / range;
        }
    });

    norm_perl
}

const CARDINAL_DELTAS: [(isize, isize); 4] = [(1, 0), (-1, 0), (0, 1), (0, -1)];

// order: lower Φ first; then closer to ocean
pub type FlowRank = (f64, f64, usize);

pub fn get_flow_rank(phi: f64, dist_ocean: f64, x: usize, y: usize, width: usize) -> FlowRank {
    (phi, dist_ocean, x + width * y)
}

// Φ = dist_ocean + lambda * norm_perlin (Φ = 0 for ocean tiles)
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
        if neighbor_rank < flow_rank
            && best_choice
                .as_ref()
                .map_or(true, |(curr_best_neighbor, _)| {
                    neighbor_rank < *curr_best_neighbor
                })
        {
            best_choice = Some((neighbor_rank, (nx, ny)));
        }
    }

    best_choice.map(|(_, coord)| coord)
}

pub fn gen_upwind_map(
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
