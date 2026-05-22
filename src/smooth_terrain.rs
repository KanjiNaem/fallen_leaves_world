use rayon::prelude::*;

use crate::helpers;

/// hermite post clamp interp.
#[inline]
fn smoothstep(edge_left: f64, edge_right: f64, x: f64) -> f64 {
    let denom = edge_right - edge_left;
    if denom.abs() <= f64::EPSILON {
        return if x < edge_left { 0.0 } else { 1.0 };
    }
    let x_to_edge_dist = ((x - edge_left) / denom).clamp(0.0, 1.0);
    x_to_edge_dist * x_to_edge_dist * (3.0 - 2.0 * x_to_edge_dist)
}

/// sugg band_span_fraction = 0.12;; share of map min–max span used as band
// pub const SUGGEST_BAND_FRAC: f64 = 0.12;
pub const SUGGEST_BAND_FRAC: f64 = 2.12;
/// sugg keep_orig_power = 1;; higher = stronger pull toward `water_level`.
pub const SUGGEST_KEEP_POWER: f64 = 2.0;
/// sugg floor for band width in heightmap units = 6.0;; avoids a vanishingly thin band on flat maps
pub const SUGGEST_MIN_BAND: f64 = 2.0;

pub fn smooth_at_lvl(
    noise_map: &Vec<Vec<f64>>,
    water_lvl: f64,
    band_span_fraction: f64,
    keep_orig_power: f64,
    min_band: f64,
) -> Vec<Vec<f64>> {
    let mut min_val = f64::INFINITY;
    let mut max_val = f64::NEG_INFINITY;
    for row in noise_map {
        for &v in row {
            min_val = min_val.min(v);
            max_val = max_val.max(v);
        }
    }
    let span = (max_val - min_val).max(f64::EPSILON);
    let frac = band_span_fraction.clamp(1e-6, 0.45);
    let band = (span * frac).max(min_band.max(f64::EPSILON));
    let power = keep_orig_power.clamp(1.0, 4.0);

    noise_map
        .into_par_iter()
        .map(|row| {
            row.into_iter()
                .map(|noise_lvl| {
                    let band_depth = ((noise_lvl - water_lvl).abs() / band).clamp(0.0, 1.0);
                    let keep_orig_dif = smoothstep(0.0, 1.0, band_depth).powf(power);
                    water_lvl + (noise_lvl - water_lvl) * keep_orig_dif
                })
                .collect()
        })
        .collect()
}
