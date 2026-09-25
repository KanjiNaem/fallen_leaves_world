// for worldsize arg, n = 10 * wsize + 2 fields in equivalence class of gp(wsize, 0),
// global data: Vec<i32>::: 
// [
//  0 <-> (n - 1): base hex noise/ 
//  n <-> (2n - 1) height adjusted hex noise
// ]
//
// idx = romb * n^2 + j*n + i || romb: [0..9] rhombi on sphere; i, j: [0..n-1] idx for a single romb
// 

use crate::{simplex_noise};

pub struct HexWorldPipelineStruct {
    pub ws: i32,
    pub master_seed: u64,
    pub start_idx_pure_noise: i32,
    pub end_idx_pure_noise: i32,
    pub start_idx_height_adj_noise: i32,
    pub end_idx_height_adj_noise: i32,
    pub global_data: Vec<f32>,
}

pub fn gen_hex_world_pipeline_struct(ws: i32, master_seed: u64) -> HexWorldPipelineStruct {
    let hex_tile_count: i32 = 10 * ws * ws + 2;
    let mut global_data: Vec<f32> = vec![0.0; (2 * hex_tile_count) as usize]; 
    let octaves = 1;
    let frequency= 1.0;
    let attenuation = 1.0;
    let amplitude = 1.0;

    let start_idx_pure_noise: i32 = 0;
    let end_idx_pure_noise: i32 = hex_tile_count - 1;
    simplex_noise::gen_octaved_simplex(&mut global_data, ws, master_seed, octaves, frequency, attenuation, amplitude, start_idx_pure_noise, end_idx_pure_noise);
    // let pure_noise_slice = &global_data[(start_idx_pure_noise as usize)..(end_idx_pure_noise as usize)];
    // println!("{:?}", pure_noise_slice);

    let lo = 0.0;
    let hi = 500.0;
    let start_idx_height_adj_noise = hex_tile_count;
    let end_idx_height_adj_noise = 2 * hex_tile_count - 1;
    simplex_noise::gen_height_adj_noise(&mut global_data, hex_tile_count, lo, hi, octaves, amplitude, attenuation, start_idx_height_adj_noise, end_idx_height_adj_noise, start_idx_pure_noise, end_idx_pure_noise);
    // let height_slice = &global_data[(start_idx_height_adj_noise as usize)..(end_idx_height_adj_noise as usize + 1)];
    // println!("{:?}",height_slice);

    HexWorldPipelineStruct { 
        ws,
        master_seed,
        start_idx_pure_noise,
        end_idx_pure_noise,
        start_idx_height_adj_noise,
        end_idx_height_adj_noise,
        global_data,
    }
}

