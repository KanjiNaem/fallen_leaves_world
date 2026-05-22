use crate::{perlin_greyscale, smooth_terrain, wind_col_grad_and_local_moisture_map};
pub struct WorldPipelineStepStruct {
    pub water_lvl: f64,
    pub noise_base: Vec<Vec<f64>>,
    pub smooth_noise: Vec<Vec<f64>>,
    pub wind_column_noise_base: Vec<Vec<f64>>,
    pub wind_column_gradient: Vec<Vec<Option<(usize, usize)>>>,
    pub moisture_map: Vec<Vec<f64>>,
}

pub fn gen_world_pipeline_step_struct(
    width: usize,
    height: usize,
    start_period: usize,
    octaves: usize,
    attenuation: f64,
    water_lvl: f64,
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
    let smooth_noise = smooth_terrain::smooth_at_lvl(
        &noise_base,
        water_lvl,
        smooth_terrain::SUGGEST_BAND_FRAC,
        smooth_terrain::SUGGEST_KEEP_POWER,
        smooth_terrain::SUGGEST_MIN_BAND,
    );
    println!("done!");

    println!("generating wind column base!");
    let wind_column_noise_base =
        perlin_greyscale::gen_single_layer_perlin_greyscale(width, height, start_period * 4);
    println!("done!");

    println!("generating flow maps");
    let (ocean_dist_map, phi_map) = wind_col_grad_and_local_moisture_map::gen_flow_rank_maps(
        width,
        height,
        &smooth_noise,
        start_period,
        water_lvl,
    );
    println!("done!");

    println!("generating wind column gradient map");
    let wind_column_gradient = wind_col_grad_and_local_moisture_map::gen_upwind_map(
        width,
        height,
        &phi_map,
        &ocean_dist_map,
        &smooth_noise,
        water_lvl,
    );
    println!("done!");

    println!("generating moisture map");
    let moisture_map = wind_col_grad_and_local_moisture_map::gen_moisture_from_flow_maps(
        width,
        height,
        &smooth_noise,
        water_lvl,
        &ocean_dist_map,
        &phi_map,
        &wind_column_gradient,
    );
    println!("done!");

    WorldPipelineStepStruct {
        water_lvl,
        noise_base,
        smooth_noise,
        wind_column_noise_base,
        wind_column_gradient,
        moisture_map,
    }
}
