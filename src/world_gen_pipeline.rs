#[allow(unused_imports)]
use crate::{
    band_influence, moisture_map, perlin_greyscale, smooth_terrain, spotted_influence,
    temperature_map, wind_col_grad_and_local_rainfall_map_old,
};
pub struct WorldPipelineStepStruct {
    pub water_lvl: f64,
    pub noise_base: Vec<Vec<f64>>,
    pub smooth_noise: Vec<Vec<f64>>,
    pub wind_column_noise_base: Vec<Vec<f64>>,
    pub wind_column_gradient: Vec<Vec<Option<(usize, usize)>>>,
    pub rainfall_map: Vec<Vec<f64>>,
    pub moisture_map: Vec<Vec<f64>>,
    pub temperature_map: Vec<Vec<f64>>,
    pub magic_influence_map: Vec<Vec<f64>>,
    pub chaos_influence_map: Vec<Vec<f64>>,
}

pub fn gen_world_pipeline_step_struct(
    width: usize,
    height: usize,
    map_z_axis: f64,
    start_period: usize,
    octaves: usize,
    attenuation: f64,
    water_lvl: f64,
    max_moisture: f64,
    magic_band_preset: band_influence::BandInfluencePresetVals,
    temp_band_noise_effect: band_influence::BandInfluencePresetVals,
    temp_preset: temperature_map::TempPresetVals,
    world_master_seed: u64,
) -> WorldPipelineStepStruct {
    assert!(width == height);

    println!("generating noise base");
    let noise_base = perlin_greyscale::gen_octaved_perlin_greyscale(
        width,
        height,
        map_z_axis,
        start_period,
        octaves,
        attenuation,
        world_master_seed,
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
    let wind_column_noise_base = perlin_greyscale::gen_single_layer_perlin_greyscale(
        width,
        height,
        map_z_axis,
        start_period * 4,
        world_master_seed + 1,
    );
    println!("done!");

    println!("generating flow maps");
    let (ocean_dist_map, phi_map) = wind_col_grad_and_local_rainfall_map_old::gen_flow_rank_maps(
        width,
        height,
        map_z_axis,
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

    println!("gen temperature map");
    let temperature_map = temperature_map::gen_temperature_map(
        width,
        height,
        &smooth_noise,
        water_lvl,
        temp_preset,
        temp_band_noise_effect,
        1.0,
        world_master_seed,
    );
    println!("done!");

    println!("gen magic noise map");
    let magic_influence_map = band_influence::gen_band_influence_map(
        width,
        height,
        &magic_band_preset,
        world_master_seed,
    );
    println!("done!");

    println!("gen chaos noise map");
    let chaos_influence_map = spotted_influence::gen_influence_map(
        width,
        height,
        4,
        400.0,
        20.0,
        5,
        50.0,
        100.0,
        world_master_seed + 1,
    );
    println!("done!");

    WorldPipelineStepStruct {
        water_lvl,
        noise_base,
        smooth_noise,
        wind_column_noise_base,
        wind_column_gradient,
        rainfall_map,
        moisture_map,
        temperature_map,
        magic_influence_map,
        chaos_influence_map,
    }
}
