use fallen_leaves_world::{
    globe_renderer, simplex_noise::NoiseParamsPresetVals,
    worldgen_pipeline::gen_hex_world_pipeline_render_struct,
};
#[kiss3d::main]
async fn main() {
    let ws = 500;
    let master_seed: u64 = 100;
    let world_height: f32 = 500.0;
    let water_lvl: i32 = 200;
    let world_noise_preset: NoiseParamsPresetVals = NoiseParamsPresetVals::DEFAULT;
    let pipeline = gen_hex_world_pipeline_render_struct(
        ws,
        world_height,
        master_seed,
        water_lvl,
        world_noise_preset,
    );
    // println!("{:?}", pipeline.global_noise_data);

    globe_renderer::render_hex_pipeline(&pipeline).await;
}
