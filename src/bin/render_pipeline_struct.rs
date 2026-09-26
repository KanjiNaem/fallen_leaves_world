use fallen_leaves_world::{globe_renderer, worldgen_pipeline::gen_hex_world_pipeline_struct};
#[kiss3d::main]
async fn main() {
    let ws = 65;
    let master_seed: u64 = 1;
    let world_height: f32 = 500.0;
    let pipeline = gen_hex_world_pipeline_struct(ws, world_height, master_seed);
    // println!("{:?}", pipeline.global_data);

    globe_renderer::render_hex_pipeline(&pipeline).await;
}