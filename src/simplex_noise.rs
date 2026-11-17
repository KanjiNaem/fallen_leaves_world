use crate::{clcg_seed_gen::Clcg};
use core::panic;

const SKEW_3D: f32 = 1.0 / 3.0;
const UNSKEW_3D: f32 = 1.0 / 6.0;
const GRAD_3D_SIMPLEX: [f32; 36] = [
    1.0, 1.0, 0.0, -1.0, 1.0, 0.0, 1.0, -1.0, 0.0, -1.0, -1.0, 0.0, 1.0, 0.0, 1.0, -1.0, 0.0, 1.0,
    1.0, 0.0, -1.0, -1.0, 0.0, -1.0, 0.0, 1.0, 1.0, 0.0, -1.0, 1.0, 0.0, 1.0, -1.0, 0.0, -1.0,
    -1.0,
]; // gradients indexed via idx * 3

pub enum NoiseParamsPresetVals {
    DEFAULT,
    SHALLOW,
    AMPLIFIED,
    ARCHIPELAGO,
}

// freqs count simplex cells across unit r;; 1 is a hemisphere scale mass
// 100+ is smaller than few hexes;; reads as static
// warp stretches mass to not be round;; detail is a fraction of world_height,
// higher HEIGHT_EXPONENT for more ocean;; higher WARP_AMPLITUDE for less circular coasts.
pub struct NoiseParams {
    continent_octaves: i32,
    continent_frequency: f32,
    continent_attenuation: f32,
    continent_gain: f32,
    detail_octaves: i32,
    detail_frequency: f32,
    detail_attenuation: f32,
    detail_weight: f32,
    warp_octaves: i32,
    warp_frequency: f32,
    warp_amplitude: f32,
    height_exponent: f32,
}

pub fn build_noise_params_preset(preset: NoiseParamsPresetVals) -> NoiseParams {
    match preset {
        NoiseParamsPresetVals::DEFAULT => NoiseParams {
            continent_octaves: 16,
            continent_frequency: 0.7,
            continent_attenuation: 0.4,
            continent_gain: 0.9,
            detail_octaves: 4,
            detail_frequency: 3.6,
            detail_attenuation: 0.5,
            detail_weight: 0.035,
            warp_octaves: 16,
            warp_frequency: 0.6,
            warp_amplitude: 0.8,
            height_exponent: 1.7,
        },
        NoiseParamsPresetVals::SHALLOW => NoiseParams {
            continent_octaves: 1,
            continent_frequency: 1.0,
            continent_attenuation: 0.4,
            continent_gain: 0.8,
            detail_octaves: 4,
            detail_frequency: 3.6,
            detail_attenuation: 0.5,
            detail_weight: 0.035,
            warp_octaves: 16,
            warp_frequency: 0.6,
            warp_amplitude: 0.8,
            height_exponent: 1.7,
        },
        NoiseParamsPresetVals::AMPLIFIED => NoiseParams {
            continent_octaves: 2,
            continent_frequency: 0.6,
            continent_attenuation: 0.7,
            continent_gain: 0.9,
            detail_octaves: 4,
            detail_frequency: 5.0,
            detail_attenuation: 0.5,
            detail_weight: 0.07,
            warp_octaves: 16,
            warp_frequency: 0.8,
            warp_amplitude: 1.2,
            height_exponent: 1.7,
        },
        NoiseParamsPresetVals::ARCHIPELAGO => NoiseParams {
            continent_octaves: 16,
            continent_frequency: 0.9,
            continent_attenuation: 0.4,
            continent_gain: 0.9,
            detail_octaves: 4,
            detail_frequency: 5.0,
            detail_attenuation: 0.5,
            detail_weight: 0.07,
            warp_octaves: 16,
            warp_frequency: 0.8,
            warp_amplitude: 1.2,
            height_exponent: 1.7,
        }
    }
}

fn fbm_max_amp(octaves: i32, amplitude: f32, attenuation: f32) -> f32 {
    (0..octaves)
        .map(|curr| amplitude * attenuation.powi(curr))
        .sum()
}

fn take_seeds(rng: &mut Clcg, n: usize) -> Vec<u64> {
    let mut seeds = vec![0u64; n];
    for seed in &mut seeds {
        *seed = rng.next_u64();
    }
    seeds
}

fn warped_unit(
    x: f32,
    y: f32,
    z: f32,
    warp_seeds: &[u64],
    warp_octaves: usize,
    warp_frequency: f32,
    warp_amplitude: f32,
) -> (f32, f32, f32) {
    let mut warp = [0.0f32; 3];
    let mut warp_freq = warp_frequency;
    let mut warp_amp = warp_amplitude;
    for w in 0..warp_octaves {
        let s = w * 3;
        warp[0] +=
            warp_amp * noise_step_3d(x * warp_freq, y * warp_freq, z * warp_freq, warp_seeds[s]);
        warp[1] += warp_amp
            * noise_step_3d(
                x * warp_freq,
                y * warp_freq,
                z * warp_freq,
                warp_seeds[s + 1],
            );
        warp[2] += warp_amp
            * noise_step_3d(
                x * warp_freq,
                y * warp_freq,
                z * warp_freq,
                warp_seeds[s + 2],
            );
        warp_freq *= 2.0;
        warp_amp *= 0.5;
    }
    let (mut px, mut py, mut pz) = (x + warp[0], y + warp[1], z + warp[2]);
    let inv_len = 1.0 / (px * px + py * py + pz * pz).sqrt().max(1.0e-8);
    px *= inv_len;
    py *= inv_len;
    pz *= inv_len;
    (px, py, pz)
}

fn fbm_at(
    x: f32,
    y: f32,
    z: f32,
    frequency: f32,
    attenuation: f32,
    amplitude: f32,
    octave_seeds: &[u64],
) -> f32 {
    let mut sum = 0.0;
    let mut freq = frequency;
    let mut amp = amplitude;
    for seed in octave_seeds {
        sum += amp * noise_step_3d(x * freq, y * freq, z * freq, *seed);
        freq *= 2.0;
        amp *= attenuation;
    }
    sum
}

pub fn gen_octaved_simplex(
    data: &mut [f32],
    pos: &[f32],
    ws: i32,
    master_seed: u64,
    octaves: i32,
    frequency: f32,
    attenuation: f32,
    amplitude: f32,
    warp_octaves: i32,
    warp_frequency: f32,
    warp_amplitude: f32,
    data_start_idx: i32,
    data_end_idx: i32,
) -> () {
    let n_cells = 10 * ws * ws + 2;
    debug_assert_eq!(pos.len(), (n_cells * 3) as usize);
    let mut rng_from_seed = Clcg::new(master_seed);
    let warp_octaves = warp_octaves.max(0) as usize;
    let octaves = octaves.max(0) as usize;
    let warp_seeds = take_seeds(&mut rng_from_seed, warp_octaves * 3);
    let octave_seeds = take_seeds(&mut rng_from_seed, octaves);

    for idx in 0..n_cells {
        let o = (idx * 3) as usize;
        let data_entry_pos = data_start_idx + idx;
        if data_entry_pos > data_end_idx {
            // not sure aboout using a panic here, however if this fucks up everything is fucked up either way so might as well kill it here!
            panic!(
                "gen_octaved_simplex tried to enter data at pos: {}, when it should only enter data until pos: {}",
                data_entry_pos, data_end_idx
            );
        }
        let (px, py, pz) = warped_unit(
            pos[o],
            pos[o + 1],
            pos[o + 2],
            &warp_seeds,
            warp_octaves,
            warp_frequency,
            warp_amplitude,
        );
        data[data_entry_pos as usize] +=
            fbm_at(px, py, pz, frequency, attenuation, amplitude, &octave_seeds);
    }
    ()
}

pub fn gen_continent_field(
    data: &mut [f32],
    pos: &[f32],
    ws: i32,
    master_seed: u64,
    lo: f32,
    world_height: f32,
    noise_params_preset_val: NoiseParamsPresetVals,
    pure_start: i32,
    pure_end: i32,
    height_start: i32,
    height_end: i32,
) -> () {
    let preset: NoiseParams = build_noise_params_preset(noise_params_preset_val);
    let NoiseParams {
        continent_octaves,
        continent_frequency,
        continent_attenuation,
        detail_octaves,
        detail_frequency,
        detail_attenuation,
        detail_weight,
        warp_octaves,
        warp_frequency,
        warp_amplitude,
        height_exponent,
        continent_gain,
    } = preset;

    let n_cells = 10 * ws * ws + 2;
    debug_assert_eq!(pos.len(), (n_cells * 3) as usize);
    let mut rng_from_seed = Clcg::new(master_seed);
    let warp_octaves = warp_octaves.max(0) as usize;
    let warp_seeds = take_seeds(&mut rng_from_seed, warp_octaves * 3);
    let continent_seeds = take_seeds(&mut rng_from_seed, continent_octaves.max(0) as usize);
    let detail_seeds = take_seeds(&mut rng_from_seed, detail_octaves.max(0) as usize);
    let broad_seeds = if continent_seeds.is_empty() {
        &continent_seeds[..]
    } else {
        &continent_seeds[..1]
    };
    let mid_seeds = if continent_seeds.len() > 1 {
        &continent_seeds[1..]
    } else {
        &continent_seeds[..0]
    };
    let broad_max = fbm_max_amp(broad_seeds.len() as i32, 1.0, 1.0).max(1.0e-5);
    let mid_max = fbm_max_amp(mid_seeds.len() as i32, 1.0, continent_attenuation).max(1.0e-5);
    let detail_max = fbm_max_amp(detail_octaves, 1.0, detail_attenuation).max(1.0e-5);
    let span = world_height - lo;

    for idx in 0..n_cells {
        let pure_at = pure_start + idx;
        let height_at = height_start + idx;
        if pure_at > pure_end || height_at > height_end || height_at < height_start {
            panic!(
                "gen_continent_field tried to enter data at pure {pure_at} / height {height_at}"
            );
        }
        let o = (idx * 3) as usize;
        let (px, py, pz) = warped_unit(
            pos[o],
            pos[o + 1],
            pos[o + 2],
            &warp_seeds,
            warp_octaves,
            warp_frequency,
            warp_amplitude,
        );
        let broad = fbm_at(px, py, pz, continent_frequency, 1.0, 1.0, broad_seeds) / broad_max;
        let mid = fbm_at(
            px,
            py,
            pz,
            continent_frequency * 2.35,
            continent_attenuation,
            1.0,
            mid_seeds,
        ) / mid_max;
        let continent = ((broad.clamp(-1.0, 1.0) + 0.45 * mid.clamp(-1.0, 1.0)) / 1.45
            * continent_gain)
            .clamp(-1.0, 1.0);
        let detail = (fbm_at(
            px,
            py,
            pz,
            detail_frequency,
            detail_attenuation,
            1.0,
            &detail_seeds,
        ) / detail_max)
            .clamp(-1.0, 1.0);
        let base = (continent * 0.5 + 0.5)
            .clamp(0.0, 1.0)
            .powf(height_exponent);
        let height_unit = (base + detail_weight * detail).clamp(0.0, 1.0);
        data[pure_at as usize] = continent + detail_weight * detail;
        data[height_at as usize] = lo + height_unit * span;
    }
    ()
}

pub fn gen_height_adj_noise(
    data: &mut [f32],
    hex_tile_count: i32,
    lo: f32,
    hi: f32,
    octaves: i32,
    amplitude: f32,
    attenuation: f32,
    height_exponent: f32,
    data_start_idx: i32,
    data_end_idx: i32,
    start_idx_pure_noise: i32,
    end_idx_pure_noise: i32,
) -> () {
    let max_amp: f32 = (0..octaves)
        .map(|curr| amplitude * attenuation.powi(curr))
        .sum();
    for curr_pure_noise_val in start_idx_pure_noise..=end_idx_pure_noise {
        let data_entry_pos = curr_pure_noise_val + hex_tile_count;
        if data_entry_pos > data_end_idx || data_entry_pos < data_start_idx {
            panic!(
                "gen_height_adj_noise tried to enter data at pos: {}, when it should only enter data until pos: {}",
                data_entry_pos, data_end_idx
            );
        }

        let temp = (data[curr_pure_noise_val as usize] / max_amp).clamp(-1.0, 1.0);
        let unit = (temp * 0.5 + 0.5).clamp(0.0, 1.0);
        data[data_entry_pos as usize] = lo + unit.powf(height_exponent) * (hi - lo);
    }
    ()
}

fn noise_step_3d(x_in: f32, y_in: f32, z_in: f32, master_seed: u64) -> f32 {
    // skew coord step
    let skew_fac = (x_in + y_in + z_in) * SKEW_3D;
    let x_sfloor = (x_in + skew_fac).floor();
    let y_sfloor = (y_in + skew_fac).floor();
    let z_sfloor = (z_in + skew_fac).floor();
    let unskewed_cell_orig = (x_sfloor + y_sfloor + z_sfloor) * UNSKEW_3D;
    let unskew_org_x0 = x_sfloor - unskewed_cell_orig;
    let unskew_org_y0 = y_sfloor - unskewed_cell_orig;
    let unskew_org_z0 = z_sfloor - unskewed_cell_orig;
    let x0 = x_in - unskew_org_x0;
    let y0 = y_in - unskew_org_y0;
    let z0 = z_in - unskew_org_z0;

    // simplical division
    let mut i1: f32 = 0.0; // first corner simplex offset
    let mut j1: f32 = 0.0;
    let mut k1: f32 = 0.0;
    let mut i2: f32 = 0.0; // second corner simplex offset
    let mut j2: f32 = 0.0;
    let mut k2: f32 = 0.0;

    if x0 >= y0 {
        if y0 >= z0 {
            i1 = 1.0; // X Y Z
            i2 = 1.0;
            j2 = 1.0;
        } else if x0 >= z0 {
            i1 = 1.0; // X Z Y
            i2 = 1.0;
            k2 = 1.0;
        } else {
            k1 = 1.0; // Z X Y
            i2 = 1.0;
            k2 = 1.0;
        }
    }

    if x0 < y0 {
        if y0 < z0 {
            k1 = 1.0; // Z Y X
            j2 = 1.0;
            k2 = 1.0;
        } else if x0 < z0 {
            j1 = 1.0; // Y Z X
            j2 = 1.0;
            k2 = 1.0;
        } else {
            j1 = 1.0; // Y X Z
            i2 = 1.0;
            j2 = 1.0;
        }
    }

    // second corner offset in xyz
    let x1: f32 = x0 - i1 + UNSKEW_3D;
    let y1: f32 = y0 - j1 + UNSKEW_3D;
    let z1: f32 = z0 - k1 + UNSKEW_3D;

    // third corner offset in xyz
    let x2: f32 = x0 - i2 + 2.0 * UNSKEW_3D;
    let y2: f32 = y0 - j2 + 2.0 * UNSKEW_3D;
    let z2: f32 = z0 - k2 + 2.0 * UNSKEW_3D;

    // fourth corner offset in xyz
    let x3: f32 = x0 - 1.0 + 3.0 * UNSKEW_3D;
    let y3: f32 = y0 - 1.0 + 3.0 * UNSKEW_3D;
    let z3: f32 = z0 - 1.0 + 3.0 * UNSKEW_3D;

    // grad selection
    let gradient_hash_0_set = funny_hash(x_sfloor, y_sfloor, z_sfloor, master_seed);
    let gradient_hash_1_set = funny_hash(x_sfloor + i1, y_sfloor + j1, z_sfloor + k1, master_seed);
    let gradient_hash_2_set = funny_hash(x_sfloor + i2, y_sfloor + j2, z_sfloor + k2, master_seed);
    let gradient_hash_3_set =
        funny_hash(x_sfloor + 1.0, y_sfloor + 1.0, z_sfloor + 1.0, master_seed);

    // kernel summation
    let mut unskewed_0: f32 = 0.6 - x0 * x0 - y0 * y0 - z0 * z0;
    let mut unskewed_1: f32 = 0.6 - x1 * x1 - y1 * y1 - z1 * z1;
    let mut unskewed_2: f32 = 0.6 - x2 * x2 - y2 * y2 - z2 * z2;
    let mut unskewed_3: f32 = 0.6 - x3 * x3 - y3 * y3 - z3 * z3;
    let mut noise_contrib_0 = 0.0;
    let mut noise_contrib_1 = 0.0;
    let mut noise_contrib_2 = 0.0;
    let mut noise_contrib_3 = 0.0;
    if unskewed_0 >= 0.0 {
        unskewed_0 *= unskewed_0;
        noise_contrib_0 = unskewed_0
            * unskewed_0
            * dot_3d(
                GRAD_3D_SIMPLEX[gradient_hash_0_set],
                GRAD_3D_SIMPLEX[gradient_hash_0_set + 1],
                GRAD_3D_SIMPLEX[gradient_hash_0_set + 2],
                x0,
                y0,
                z0,
            )
    }

    if unskewed_1 >= 0.0 {
        unskewed_1 *= unskewed_1;
        noise_contrib_1 = unskewed_1
            * unskewed_1
            * dot_3d(
                GRAD_3D_SIMPLEX[gradient_hash_1_set],
                GRAD_3D_SIMPLEX[gradient_hash_1_set + 1],
                GRAD_3D_SIMPLEX[gradient_hash_1_set + 2],
                x1,
                y1,
                z1,
            );
    }

    if unskewed_2 >= 0.0 {
        unskewed_2 *= unskewed_2;
        noise_contrib_2 = unskewed_2
            * unskewed_2
            * dot_3d(
                GRAD_3D_SIMPLEX[gradient_hash_2_set],
                GRAD_3D_SIMPLEX[gradient_hash_2_set + 1],
                GRAD_3D_SIMPLEX[gradient_hash_2_set + 2],
                x2,
                y2,
                z2,
            )
    }

    if unskewed_3 >= 0.0 {
        unskewed_3 *= unskewed_3;
        noise_contrib_3 = unskewed_3
            * unskewed_3
            * dot_3d(
                GRAD_3D_SIMPLEX[gradient_hash_3_set],
                GRAD_3D_SIMPLEX[gradient_hash_3_set + 1],
                GRAD_3D_SIMPLEX[gradient_hash_3_set + 2],
                x3,
                y3,
                z3,
            )
    }

    32.0 * (noise_contrib_0 + noise_contrib_1 + noise_contrib_2 + noise_contrib_3)
}

fn funny_hash(x: f32, y: f32, z: f32, seed: u64) -> usize {
    let mut funny: i32 = (x * 374761393.0 + y * 668265263.0 + z * 1446898217.0) as i32;
    funny ^= seed as i32;
    funny ^= (seed >> 32) as i32;
    funny = (funny ^ (funny >> 13)).wrapping_mul(1274126177);
    funny ^= funny >> 16;
    ((funny as u32 as usize) % 12) * 3
}

fn dot_3d(x0: f32, y0: f32, z0: f32, x1: f32, y1: f32, z1: f32) -> f32 {
    x0 * x1 + y0 * y1 + z0 * z1
}
