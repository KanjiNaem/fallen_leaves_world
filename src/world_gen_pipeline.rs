use crate::perlin_greyscale;
use crate::island_mask;
// use crate::

// elevation noise --> island mask

pub struct WorldPipelineStepStruct {
    pub water_lvl: f64,
    pub noise_base: Vec<Vec<f64>>,
    pub smooth_noise: Vec<Vec<f64>>,
}

pub fn gen_world_pipeline_step_struct(width: usize, height: usize, start_period: usize, octaves: usize, attenuation: f64, water_lvl: f64) -> WorldPipelineStepStruct {

    let noise_base = perlin_greyscale::gen_octaved_perlin_greyscale(width, height, start_period, octaves, attenuation);
    let smooth_noise = island_mask::smooth_at_lvl(&noise_base, water_lvl, island_mask::SUGGEST_BAND_FRAC, island_mask::SUGGEST_KEEP_POWER, island_mask::SUGGEST_MIN_BAND);

    WorldPipelineStepStruct { 
        water_lvl,
        noise_base,
        smooth_noise 
    }
}