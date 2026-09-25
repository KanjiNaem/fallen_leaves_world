use fallen_leaves_world::{globe_renderer, worldgen_pipeline::gen_hex_world_pipeline_struct};
#[kiss3d::main]
async fn main() {
    let ws = 3;
    let master_seed = 1;
    let pipeline = gen_hex_world_pipeline_struct(ws, master_seed);
    // println!("{:?}", pipeline.global_data);

    globe_renderer::render_hex_pipeline(&pipeline).await;
}