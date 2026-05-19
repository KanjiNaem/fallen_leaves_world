use rand::{Rng, thread_rng};
use rayon::prelude::*;
use std::f64::consts::PI;

pub type Vec2d = (f64, f64);

#[inline]
fn floor_div(a: usize, b: usize) -> usize {
    (a as f64 / b as f64).floor() as usize
}

#[inline]
fn dot(x1: f64, x2: f64, y1: f64, y2: f64) -> f64 {
    x1 * x2 + y1 * y2
}

#[inline]
fn lin_interp(a: f64, b: f64, t: f64) -> f64 {
    a * (1.0 - t) + b * t
}

#[inline]
fn smooth(x: f64) -> f64 {
    6.0 * x.powf(5.0) - 15.0 * x.powf(4.0) + 10.0 * x.powf(3.0)
}

fn rand_vec2d() -> Vec2d {
    let mut rng = thread_rng();
    let angle = rng.gen_range(0.0..(2.0 * PI));
    (angle.cos(), angle.sin())
}

fn get_perlin_value(x: usize, y: usize, perlin_vectors: &Vec<Vec<Vec2d>>, period: usize) -> f64 {
    // cell coord
    let cell_x = floor_div(x, period);
    let cell_y = floor_div(y, period);

    // norm rel coords in cell
    let mut relative_x: f64 = (x as f64 - cell_x as f64 * period as f64) / period as f64;
    let mut relative_y: f64 = (y as f64 - cell_y as f64 * period as f64) / period as f64;

    relative_x = smooth(relative_x);
    relative_y = smooth(relative_y);

    let top_left_gradient = perlin_vectors[cell_y][cell_x];
    let top_right_gradient = perlin_vectors[cell_y][cell_x + 1];
    let bottom_left_gradient = perlin_vectors[cell_y + 1][cell_x];
    let bottom_right_gradient = perlin_vectors[cell_y + 1][cell_x + 1];

    let top_left_contribution = dot(
        top_left_gradient.0,
        top_left_gradient.1,
        relative_x,
        relative_y,
    );

    let top_right_contribution = dot(
        top_right_gradient.0,
        top_right_gradient.1,
        relative_x - 1.0,
        relative_y,
    );

    let bottom_left_contribution = dot(
        bottom_left_gradient.0,
        bottom_left_gradient.1,
        relative_x,
        relative_y - 1.0,
    );

    let bottom_rigth_contribution = dot(
        bottom_right_gradient.0,
        bottom_right_gradient.1,
        relative_x - 1.0,
        relative_y - 1.0,
    );

    let top_lin_interp = lin_interp(top_left_contribution, top_right_contribution, relative_x);
    let bottom_lin_interp = lin_interp(
        bottom_left_contribution,
        bottom_rigth_contribution,
        relative_x,
    );
    let final_value = lin_interp(top_lin_interp, bottom_lin_interp, relative_y);

    final_value / (2.0_f64.sqrt() / 2.0)
}

pub fn gen_single_layer_perlin_greyscale(
    width: usize,
    height: usize,
    period: usize,
) -> Vec<Vec<f64>> {
    assert!(period >= 1);

    let lattice_w = ((width.saturating_sub(1)) as f64 / period as f64).floor() as usize + 2;
    let lattice_h = ((height.saturating_sub(1)) as f64 / period as f64).floor() as usize + 2;
    let lattice_w = lattice_w.max(2);
    let lattice_h = lattice_h.max(2);
    let perlin_vectors: Vec<Vec<Vec2d>> = (0..lattice_h)
        .map(|_| (0..lattice_w).map(|_| rand_vec2d()).collect())
        .collect();

    let mut result_px_grid: Vec<Vec<f64>> = vec![vec![0.0; width]; height];
    result_px_grid
        .par_iter_mut()
        .enumerate()
        .for_each(|(y, row)| {
            for x in 0..width {
                let perlin_value = get_perlin_value(x, y, &perlin_vectors, period);
                row[x] = 128.0 + 128.0 * perlin_value;
            }
        });

    result_px_grid
}

pub fn gen_octaved_perlin_greyscale(
    width: usize,
    height: usize,
    start_period: usize,
    octaves: usize,
    attenuation: f64,
) -> Vec<Vec<f64>> {
    assert!(octaves > 0);
    assert!(start_period >= 1);

    let mut final_grid = vec![vec![0.0f64; width]; height];

    for octave in 0..octaves {
        let period = (start_period / 2_usize.pow(octave as u32)).max(1);
        let octave_attenuation = attenuation.powi(octave as i32);

        let lattice_w = ((width.saturating_sub(1)) as f64 / period as f64).floor() as usize + 2;
        let lattice_h = ((height.saturating_sub(1)) as f64 / period as f64).floor() as usize + 2;
        let lattice_w = lattice_w.max(2);
        let lattice_h = lattice_h.max(2);
        let perlin_vectors: Vec<Vec<Vec2d>> = (0..lattice_h)
            .map(|_| (0..lattice_w).map(|_| rand_vec2d()).collect())
            .collect();

        final_grid.par_iter_mut().enumerate().for_each(|(y, row)| {
            for x in 0..width {
                let perl_val = get_perlin_value(x, y, &perlin_vectors, period);
                row[x] += perl_val * octave_attenuation;
            }
        });
    }

    let max_value: f64 = (0..octaves).map(|i| attenuation.powi(i as i32)).sum();

    let mut result_px_grid: Vec<Vec<f64>> = vec![vec![0.0; width]; height];
    result_px_grid
        .par_iter_mut()
        .zip(final_grid.par_iter())
        .for_each(|(out_row, acc_row)| {
            for x in 0..width {
                let n = acc_row[x] / max_value;
                out_row[x] = 128.0 + 128.0 * n;
            }
        });

    result_px_grid
}
