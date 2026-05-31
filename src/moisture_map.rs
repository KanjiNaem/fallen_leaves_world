use crate::helpers;

use rayon::prelude::*;
use std::sync::Mutex;

// non directional rainfall, manhattan dist based moisture radius around water bodies
// base moisture 100 regardless of size
pub fn gen_moisture_map(
    width: usize,
    height: usize,
    terrain_map: &Vec<Vec<f64>>,
    water_lvl: f64,
    max_moisture: f64,
) -> Vec<Vec<f64>> {
    let base_moisture = 100.0;
    let map_len = width * height;
    let mut terrain_flat = Vec::with_capacity(map_len);
    for row in terrain_map {
        terrain_flat.extend_from_slice(row);
    }
    let water_body_groups = helpers::group_water_bodies(width, height, &terrain_map, water_lvl);

    let deposit_sum = Mutex::new(vec![0.0f64; map_len]);

    water_body_groups.par_iter().for_each(|(_, curr_group)| {
        let curr_size = curr_group.len();
        let infl_rad = get_influence_radius(width, height, curr_size);
        let curr_shore =
            helpers::get_shore_adj_for_body(width, height, curr_group, &terrain_map, water_lvl);

        if curr_shore.is_empty() {
            return;
        }

        let (dist_map, min_x, min_y) =
            helpers::gen_manhattan_dist_to_shore(width, height, &curr_shore, infl_rad);
        let max_x = min_x + dist_map[0].len() - 1;
        let max_y = min_y + dist_map.len() - 1;

        let updates: Vec<(usize, f64)> = (min_y..=max_y)
            .into_par_iter()
            .flat_map(|y| {
                let dist_row = &dist_map[y - min_y];
                let row_off = y * width;
                let terrain_row = &terrain_flat[row_off..(row_off + width)];
                (min_x..=max_x)
                    .filter_map(|x| {
                        if terrain_row[x] <= water_lvl {
                            return None;
                        }

                        let curr_rad = dist_row[x - min_x];
                        if curr_rad >= infl_rad {
                            return None;
                        }

                        let curr_percentile =
                            get_influence_moisture_percentile(curr_rad as f64, infl_rad as f64);
                        let deposit = curr_percentile * base_moisture;

                        Some((row_off + x, deposit))
                    })
                    .collect::<Vec<_>>()
            })
            .collect();

        let mut sum_guard = deposit_sum.lock().unwrap();
        for (idx, deposit) in updates {
            sum_guard[idx] += deposit;
        }
    });

    let moisture_flat: Vec<f64> = deposit_sum
        .into_inner()
        .unwrap()
        .into_iter()
        .map(|s| s.min(max_moisture))
        .collect();

    let moisture_map: Vec<Vec<f64>> = moisture_flat
        .chunks_exact(width)
        .map(|row| row.to_vec())
        .collect();

    return moisture_map;
}

fn get_influence_radius(width: usize, height: usize, water_body_size: usize) -> usize {
    let map_size = width * height;
    if water_body_size >= map_size / 8 {
        // println!("1");
        return width;
    } else if water_body_size >= map_size / 60 {
        // println!("2");
        return width / 2;
    } else if water_body_size >= map_size / 120 {
        // println!("8");
        return width / 3;
    } else if water_body_size >= map_size / 300 {
        // println!("16");
        return width / 4;
    } else if water_body_size >= map_size / 700 {
        // println!("32");
        return width / 16;
    } else {
        // println!("64");
        return width / 32;
    }
}

fn get_influence_moisture_percentile(curr_rad: f64, total_rad: f64) -> f64 {
    if curr_rad <= total_rad / 4.0 {
        return 1.0;
    } else if curr_rad <= total_rad / 3.0 {
        return 0.75;
    } else if curr_rad <= total_rad / 2.0 {
        return 0.5;
    } else if curr_rad <= total_rad / 1.5 {
        return 0.25;
    } else if curr_rad <= total_rad / 1.25 {
        return 0.15;
    } else {
        return 0.1;
    }
}
