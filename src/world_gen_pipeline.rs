use crate::{
    moisture_map, perlin_greyscale, smooth_terrain, wind_col_grad_and_local_rainfall_map_old,
};
pub struct WorldPipelineStepStruct {
    pub water_lvl: f64,
    pub noise_base: Vec<Vec<f64>>,
    pub smooth_noise: Vec<Vec<f64>>,
    pub wind_column_noise_base: Vec<Vec<f64>>,
    pub wind_column_gradient: Vec<Vec<Option<(usize, usize)>>>,
    pub rainfall_map: Vec<Vec<f64>>,
    pub moisture_map: Vec<Vec<f64>>,
}

pub fn gen_world_pipeline_step_struct(
    width: usize,
    height: usize,
    start_period: usize,
    octaves: usize,
    attenuation: f64,
    water_lvl: f64,
    max_moisture: f64,
) -> WorldPipelineStepStruct {
    assert!(width == height);

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
    let (ocean_dist_map, phi_map) = wind_col_grad_and_local_rainfall_map_old::gen_flow_rank_maps(
        width,
        height,
        &smooth_noise,
        start_period,
        water_lvl,
    );
    println!("done!");

    println!("generating wind column gradient map");
    let wind_column_gradient = wind_col_grad_and_local_rainfall_map_old::gen_upwind_map(
        width,
        height,
        &phi_map,
        &ocean_dist_map,
        &smooth_noise,
        water_lvl,
    );
    println!("done!");

    println!("generating local rainfall map");
    let rainfall_map = wind_col_grad_and_local_rainfall_map_old::gen_rainfall_from_flow_maps(
        width,
        height,
        &smooth_noise,
        water_lvl,
        &ocean_dist_map,
        &phi_map,
        &wind_column_gradient,
    );
    println!("done!");

    println!("gen new moisture map");
    let moisture_map =
        moisture_map::gen_moisture_map(width, height, &smooth_noise, water_lvl, max_moisture);
    println!("done!");

    WorldPipelineStepStruct {
        water_lvl,
        noise_base,
        smooth_noise,
        wind_column_noise_base,
        wind_column_gradient,
        rainfall_map,
        moisture_map,
    }
}
