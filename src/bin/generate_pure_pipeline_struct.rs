use fallen_leaves_world::{
    simplex_noise::NoiseParamsPresetVals, worldgen_pipeline::gen_hex_world_pipeline_pure_struct,
};

fn main() {
    let ws = 5;
    let master_seed: u64 = 10;
    let world_height: f32 = 500.0;
    let water_lvl: i32 = 200;
    let world_noise_preset: NoiseParamsPresetVals = NoiseParamsPresetVals::DEFAULT;
    let pipeline = gen_hex_world_pipeline_pure_struct(
        ws,
        world_height,
        master_seed,
        water_lvl,
        world_noise_preset,
    );
    println!("{:?}", pipeline.global_noise_data);
}
