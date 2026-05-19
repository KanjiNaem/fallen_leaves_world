use crate::island_mask;
use crate::moisture_rainfall_mask;
use crate::perlin_greyscale;

// elevation noise --> island mask --> moisture rainfall mask

pub struct WorldPipelineStepStruct {
    pub water_lvl: f64,
    pub noise_base: Vec<Vec<f64>>,
    pub smooth_noise: Vec<Vec<f64>>,
    pub moisture_map: Vec<Vec<f64>>,
}

pub fn gen_world_pipeline_step_struct(
    width: usize,
    height: usize,
    start_period: usize,
    octaves: usize,
    attenuation: f64,
    water_lvl: f64,
    rain_coeff: f64,
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

    println!("generating moisture map");
    let moisture_map =
        moisture_rainfall_mask::gen_moisture_rainfall_mask(&smooth_noise, water_lvl, rain_coeff);
    println!("done!");

    WorldPipelineStepStruct {
        water_lvl,
        noise_base,
        smooth_noise,
        moisture_map,
    }
}
