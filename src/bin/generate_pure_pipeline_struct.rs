use fallen_leaves_world::{worldgen_pipeline::gen_hex_world_pipeline_pure_struct};

fn main() {
    let ws = 65; // 65 is a good target size for sim stuffs;; 150 is too large for gameplay but maybe cool for sim stuff
    let master_seed: u64 = 10;
    let world_height: f32 = 500.0;
    let pipeline = gen_hex_world_pipeline_pure_struct(ws, world_height, master_seed);
    println!("{:?}", pipeline.global_noise_data);
}
