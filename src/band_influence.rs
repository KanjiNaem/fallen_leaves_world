use crate::{clcg_seed_gen, perlin_greyscale};
use rayon::prelude::*;

// line_period: perlin cell size for line noise;  larger → smoother, longer curves before wiggle
// warp_period: perlin cell size for two warp layers; larger → slower broader distortion
// warp_amplitude: how far each sample is shifted before reading line field:
//// wx = x + warp_amplitude * warp_x[y][x]
//// wy = y + warp_y[y][x]
//// warp_x / warp_y are roughly [-1, 1]; effective shift is ~±warp_amplitude; bigger → more bend and swirl;

// band: half of width of ridge in noise units; perlin sample n is ~[-1, 1], ;trength is: (1 - |n| / band)² × low_max when |n| < band, else 0.
// small band: thin fill
// large band: thick ribbons; near 1.0 almost everything gets some value
// low_max: peak strength on the ridges; ceiling for band layer only, not the whole map

pub fn gen_band_influence_map(
    width: usize,
    height: usize,
    line_period: usize,
    warp_period: usize,
    warp_amplitude: f64,
    band: f64,
    low_max: f64,
    p_peaks: usize,
    peak_radius: f64,
    peak_max: f64,
    world_master_seed: u64,
) -> Vec<Vec<f64>> {
    let line_noise = perlin_greyscale::gen_single_layer_perlin_greyscale(
        width,
        height,
        1.0,
        line_period.max(1),
        world_master_seed,
    );
    let warp_x = perlin_greyscale::gen_single_layer_perlin_greyscale(
        width,
        height,
        1.0,
        warp_period.max(1),
        world_master_seed + 1,
    );
    let warp_y = perlin_greyscale::gen_single_layer_perlin_greyscale(
        width,
        height,
        1.0,
        warp_period.max(1),
        world_master_seed + 2,
    );

    let mut map: Vec<Vec<f64>> = (0..height)
        .into_par_iter()
        .map(|y| {
            (0..width)
                .map(|x| {
                    let warped_x = x as f64 + warp_amplitude * warp_x[y][x];
                    let warped_y = y as f64 + warp_amplitude * warp_y[y][x];
                    let val = sample_bilinear(&line_noise, warped_x, warped_y);
                    ridge_strength(val, band, low_max)
                })
                .collect()
        })
        .collect();

    let mut rng = clcg_seed_gen::Clcg::new(world_master_seed + 3);
    for _ in 0..p_peaks {
        let rng_x = rng.next_u32() % width as u32;
        let rng_y = rng.next_u32() % height as u32;
        stamp_disk(&mut map, rng_x as i32, rng_y as i32, peak_radius, peak_max);
    }
    map
}

#[inline]
fn ridge_strength(val: f64, band: f64, low_max: f64) -> f64 {
    let ridge_str = 1.0 - val.abs() / band;
    if ridge_str <= 0.0 {
        0.0
    } else {
        low_max * ridge_str * ridge_str
    }
}

fn sample_bilinear(grid: &[Vec<f64>], warp_x: f64, warp_y: f64) -> f64 {
    let width = grid[0].len();
    let height = grid.len();
    if width == 0 || height == 0 {
        return 0.0;
    }
    let c_warp_x = warp_x.clamp(0.0, (width - 1) as f64);
    let c_warp_y = warp_y.clamp(0.0, (height - 1) as f64);
    let x0 = c_warp_x.floor() as usize;
    let y0 = c_warp_y.floor() as usize;
    let x1 = (x0 + 1).min(width - 1);
    let y1 = (y0 + 1).min(height - 1);
    let decimal_x = c_warp_x - x0 as f64;
    let decimal_y = c_warp_y - y0 as f64;
    let next00 = grid[y0][x0];
    let next10 = grid[y0][x1];
    let next01 = grid[y1][x0];
    let next11 = grid[y1][x1];
    let top = next00 * (1.0 - decimal_x) + next10 * decimal_x;
    let bottom = next01 * (1.0 - decimal_x) + next11 * decimal_x;
    top * (1.0 - decimal_y) + bottom * decimal_y
}

fn stamp_disk(map: &mut [Vec<f64>], x: i32, y: i32, rad: f64, strength: f64) {
    let rad_i = rad.ceil() as i32;
    for dy in -rad_i..=rad_i {
        for dx in -rad_i..=rad_i {
            let dist = ((dx * dx + dy * dy) as f64).sqrt();
            if dist > rad {
                continue;
            }
            let t = 1.0 - dist / rad;
            let v = strength * t * t;
            let x_pos = x + dx;
            let y_pos = y + dy;
            if x_pos < 0
                || y_pos < 0
                || x_pos as usize >= map[0].len()
                || y_pos as usize >= map.len()
            {
                continue;
            }
            let x = x_pos as usize;
            let y = y_pos as usize;
            map[y][x] = map[y][x].max(v);
        }
    }
}
