use crate::{island_mask, wind_column_gradient_map, perlin_greyscale};
pub struct WorldPipelineStepStruct {
    pub water_lvl: f64,
    pub noise_base: Vec<Vec<f64>>,
    pub smooth_noise: Vec<Vec<f64>>,
    pub wind_column_noise_base: Vec<Vec<f64>>,
    pub wind_column_gradient: Vec<Vec<Option<(usize, usize)>>>,
    // pub moisture_map: Vec<Vec<f64>>,
}

pub fn gen_world_pipeline_step_struct(
    width: usize,
    height: usize,
    start_period: usize,
    octaves: usize,
    attenuation: f64,
    water_lvl: f64,
    _rain_coeff: f64,
) -> WorldPipelineStepStruct {
    println!("generating noise base");
    let noise_base = perlin_greyscale::gen_octaved_perlin_greyscale(
        width,
        height,
        start_period,
        octaves,
        attenuation,
    );
    println!("done!");

    println!("smoothing shore lines of noise base");
    let smooth_noise = island_mask::smooth_at_lvl(
        &noise_base,
        water_lvl,
        island_mask::SUGGEST_BAND_FRAC,
        island_mask::SUGGEST_KEEP_POWER,
        island_mask::SUGGEST_MIN_BAND,
    );
    println!("done!");

    println!("generating wind column base!");
    let wind_column_noise_base = perlin_greyscale::gen_single_layer_perlin_greyscale(width, height, start_period * 4);
    println!("done!");

    println!("generating wind column gradient map");
    let wind_column_gradient =  wind_column_gradient_map::gen_wind_column_gradient_map(
        width, 
        height,
        &smooth_noise,
        start_period,
        water_lvl
    );
    println!("done!");

    WorldPipelineStepStruct {
        water_lvl,
        noise_base,
        smooth_noise,
        wind_column_noise_base,
        wind_column_gradient,
    }
}
