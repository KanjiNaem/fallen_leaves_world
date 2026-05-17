use rayon::prelude::*;

#[inline]
fn smoothstep_01(norm_height: f64) -> f64 {
    let norm_height = norm_height.clamp(0.0, 1.0);
    norm_height * norm_height * (3.0 - 2.0 * norm_height)
}

pub fn apply_terrace_redistrib(
    noise_map: &Vec<Vec<f64>>,
    steps: f64,
    smooth: bool,
) -> Vec<Vec<f64>> {
    assert!(steps >= 1.0);

    noise_map.to_vec().par_iter_mut().for_each(|row| {
        for vec in row.iter_mut() {
            let norm_height = (*vec / 255.0).clamp(0.0, 1.0);
            let terrace_intervals = norm_height * steps;
            let base = terrace_intervals.floor();
            let frac = terrace_intervals - base;
            let shaped = if smooth {
                base + smoothstep_01(frac)
            } else {
                base
            };
            let out_norm_height = (shaped / steps).clamp(0.0, 1.0);
            *vec = out_norm_height * 255.0;
        }
    });

    noise_map.to_vec()
}
