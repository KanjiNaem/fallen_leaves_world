use rayon::prelude::*;

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
pub const LARGE_RANGE_BAND_FRAC: f64 = 20.0;
/// sugg keep_orig_power = 1;; higher = stronger pull toward `water_level`.
pub const SUGGEST_KEEP_POWER: f64 = 2.0;
pub const LARGE_LEVEL_PULL: f64 = 100.0;
/// sugg floor for band width in heightmap units = 6.0;; avoids a vanishingly thin band on flat maps
pub const SUGGEST_MIN_BAND: f64 = 2.0;
/// Underwater shore band as a fraction of the land band;; smaller = ocean depth preserved sooner.
pub const UNDERWATER_BAND_FRAC: f64 = 0.18;
/// Pull strength below water;; lower retains more original seabed depth in the transition zone.
pub const UNDERWATER_KEEP_POWER: f64 = 0.65;

pub fn smooth_at_lvl(
    noise_map: &Vec<Vec<f64>>,
    smooth_at_lvl: f64,
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
    let underwater_band = (band * UNDERWATER_BAND_FRAC.clamp(0.02, 1.0)).max(f64::EPSILON);
    let underwater_power = UNDERWATER_KEEP_POWER.clamp(0.25, 4.0);

    noise_map
        .into_par_iter()
        .map(|row| {
            row.into_iter()
                .map(|noise_lvl| {
                    let diff = noise_lvl - smooth_at_lvl;
                    let keep_orig_dif = if diff >= 0.0 {
                        let band_depth = (diff / band).clamp(0.0, 1.0);
                        smoothstep(0.0, 1.0, band_depth).powf(power)
                    } else {
                        // Narrower, weaker smoothing below water so oceans keep their depth.
                        let band_depth = (diff.abs() / underwater_band).clamp(0.0, 1.0);
                        smoothstep(0.0, 1.0, band_depth).powf(underwater_power)
                    };
                    smooth_at_lvl + diff * keep_orig_dif
                })
                .collect()
        })
        .collect()
}
