#[allow(unused_imports)]
use crate::{
    chaos_influence_map, magic_influence_map, moisture_map, perlin_greyscale, smooth_terrain,
    wind_col_grad_and_local_rainfall_map_old,
};
pub struct WorldPipelineStepStruct {
    pub water_lvl: f64,
    pub noise_base: Vec<Vec<f64>>,
    pub smooth_noise: Vec<Vec<f64>>,
    pub wind_column_noise_base: Vec<Vec<f64>>,
    pub wind_column_gradient: Vec<Vec<Option<(usize, usize)>>>,
    pub rainfall_map: Vec<Vec<f64>>,
    pub moisture_map: Vec<Vec<f64>>,
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
        1,
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
        2,
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

    println!("gen magic noise map");
    let magic_influence_map_base =
        perlin_greyscale::gen_octaved_perlin_greyscale(width, height, 500.0, 1000, 3, 0.9, 3);
    let magic_influence_map = smooth_terrain::smooth_at_lvl(
        &magic_influence_map_base,
        50.0,
        smooth_terrain::LARGE_RANGE_BAND_FRAC,
        smooth_terrain::SUGGEST_KEEP_POWER,
        smooth_terrain::SUGGEST_MIN_BAND,
    );
    println!("done!");

    println!("gen chaos noise map");
    let chaos_influence_map_base =
        perlin_greyscale::gen_octaved_perlin_greyscale(width, height, 500.0, 1000, 3, 0.9, 4);
    let chaos_influence_map = smooth_terrain::smooth_at_lvl(
        &chaos_influence_map_base,
        50.0,
        smooth_terrain::LARGE_LEVEL_PULL,
        smooth_terrain::SUGGEST_KEEP_POWER,
        smooth_terrain::SUGGEST_MIN_BAND,
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
        magic_influence_map,
        chaos_influence_map,
    }
}
