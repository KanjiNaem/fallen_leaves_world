use crate::band_influence;
use rayon::prelude::*;

// lowest lower_bound for img gen
pub const TEMP_DISPLAY_MIN: f64 = -25.0;
// highest upper_bound for img gen
pub const TEMP_DISPLAY_MAX: f64 = 75.0;

pub fn gen_temperature_map(
    width: usize,
    height: usize,
    terrain_map: &Vec<Vec<f64>>,
    water_lvl: f64,
    temp_preset: TempPresetVals,
    band_preset: band_influence::BandInfluencePresetVals,
    band_influence_coef: f64,
    world_master_seed: u64,
) -> Vec<Vec<f64>> {
    let preset: TempPreset = TempPreset::new(&temp_preset);
    let TempPreset {
        height_weight_coef,
        base_land_temp,
        base_water_temp,
        lower_bound,
        upper_bound,
    } = preset;

    let band_temp_noise_map =
        band_influence::gen_band_influence_map(width, height, &band_preset, world_master_seed);
    let highest_influence = band_influence::BandInfluencePreset::bound(
        &band_influence::BandInfluencePreset::new(&band_preset, width),
    );

    let norm_band_temp_noise_map: Vec<Vec<f64>> = band_temp_noise_map
        .par_iter()
        .map(|row| {
            row.par_iter()
                .map(|&curr_val| curr_val / highest_influence)
                .collect()
        })
        .collect();

    let temp_map: Vec<Vec<f64>> = (0..height)
        .into_par_iter()
        .map(|y| {
            (0..width)
                .map(|x| {
                    if terrain_map[y][x] <= water_lvl {
                        let base_temp = base_water_temp;
                        let relative_height = terrain_map[y][x] - water_lvl;
                        ((base_temp - (height_weight_coef * relative_height))
                            + 10.0 * (band_influence_coef * norm_band_temp_noise_map[y][x]))
                            .clamp(lower_bound, upper_bound)
                    } else {
                        let base_temp = base_land_temp;
                        let relative_height = water_lvl - terrain_map[y][x];
                        ((base_temp - (height_weight_coef * relative_height))
                            * (1.0 + (band_influence_coef * norm_band_temp_noise_map[y][x])))
                            .clamp(lower_bound, upper_bound)
                    }
                })
                .collect()
        })
        .collect();
    temp_map
}

pub enum TempPresetVals {
    Low,
    Middle,
    High,
    VeryHigh,
}
pub struct TempPreset {
    height_weight_coef: f64,
    base_land_temp: f64,
    base_water_temp: f64,
    lower_bound: f64,
    upper_bound: f64,
}

impl TempPreset {
    pub fn new(preset: &TempPresetVals) -> Self {
        match preset {
            TempPresetVals::Low => Self {
                height_weight_coef: 0.05,
                base_land_temp: 18.0,
                base_water_temp: 12.0,
                lower_bound: -25.0,
                upper_bound: 50.0,
            },
            TempPresetVals::Middle => Self {
                height_weight_coef: 0.025,
                base_land_temp: 22.0,
                base_water_temp: 16.0,
                lower_bound: -15.0,
                upper_bound: 60.0,
            },
            TempPresetVals::High => Self {
                height_weight_coef: 0.015,
                base_land_temp: 26.00,
                base_water_temp: 19.0,
                lower_bound: -5.0,
                upper_bound: 70.0,
            },
            TempPresetVals::VeryHigh => Self {
                height_weight_coef: 0.001,
                base_land_temp: 30.0,
                base_water_temp: 24.0,
                lower_bound: 0.0,
                upper_bound: 75.0,
            },
        }
    }
}
