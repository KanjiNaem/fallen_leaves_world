use fallen_leaves_world::{globe_renderer, worldgen_pipeline::gen_hex_world_pipeline_render_struct};
#[kiss3d::main]
async fn main() {
    let ws = 100; // 65 is a good target size for sim stuffs;; 150 is too large for gameplay but maybe cool for sim stuff
    let master_seed: u64 = 10;
    let world_height: f32 = 500.0;
    let pipeline = gen_hex_world_pipeline_render_struct(ws, world_height, master_seed);
    // println!("{:?}", pipeline.global_noise_data);

    globe_renderer::render_hex_pipeline(&pipeline).await;
}
