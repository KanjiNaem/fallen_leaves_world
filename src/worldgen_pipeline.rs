// n = 10 * ws^2 + 2 geodesic vertices of a class-I icosa (GP(ws, 0)).
// last two slots are the poles; remaining ids are the other lattice points.
// 
// pub unit_sphere_center_pos: Vec<f32>
//[
//  n*3: xyz with 3-step-stride
//]
//
// pub neighbor_ids: Vec<i32>
//[
//  n*6: neighbor ids of idx with 6-step-stride;; last unused for missing neighbor of pents
//]
//
// pub neighbor_cnt: Vec<u8>
//[
//  n: count neighbor amnt of idx;; 5 or 6
//]
//
// pub hex_verts: Vec<f32>
//[
//  n*7*3: verts of hexes (last entries unused for pents);; (center + 6 corners) * xyz 
//]
//
// pub norm_tan_height: Vec<f32>
//[
//  n*7: tan height in 0..1 
//]
//
// pub tri_faces_idxs: Vec<u32>
//[
//  n*6*3: 6 tri idxs in hex verts;; -last unused for missing tri of pent
//  vert v is at hex_verts[v*3..v*3+3] for v = idx*7 + local
//]
//
// global data: Vec<i32>
// [
//  0 <-> (n - 1): base hex noise/ 
//  n <-> (2n - 1) height adjusted hex noise
// ]
//

use crate::{helpers, simplex_noise};

pub struct HexWorldPipelineRenderStruct {
    pub ws: i32,
    pub n: i32,
    pub world_height: f32,
    pub master_seed: u64,
    pub start_idx_pure_noise: i32,
    pub end_idx_pure_noise: i32,
    pub start_idx_height_adj_noise: i32,
    pub end_idx_height_adj_noise: i32,
    pub cell_unit_sphere_pos: Vec<f32>,
    pub neighbor_ids: Vec<i32>,
    pub neighbor_cnt: Vec<u8>,
    pub hex_verts: Vec<f32>,
    pub norm_tan_height: Vec<f32>,
    pub tri_faces_idxs: Vec<u32>,
    pub global_noise_data: Vec<f32>,
}

pub fn gen_hex_world_pipeline_struct(ws: i32, world_height: f32, master_seed: u64) -> HexWorldPipelineRenderStruct {
    let hex_tile_count: i32 = 10 * ws * ws + 2;
    let mut global_noise_data: Vec<f32> = vec![0.0; (2 * hex_tile_count) as usize]; 
    let octaves = 8;
    let frequency= 1.0;
    let attenuation = 1.0;
    let amplitude = 1.0;

    // pure noise 0..(n-1)
    let start_idx_pure_noise: i32 = 0;
    let end_idx_pure_noise: i32 = hex_tile_count - 1;
    let lo = 0.0;
    let start_idx_height_adj_noise = hex_tile_count;
    let end_idx_height_adj_noise = 2 * hex_tile_count - 1;

    let mut cell_unit_sphere_pos: Vec<f32> = vec![0.0; (3 * hex_tile_count) as usize];
    helpers::gen_cell_unit_sphere_pos_data(&mut cell_unit_sphere_pos, hex_tile_count, ws);

    simplex_noise::gen_octaved_simplex(&mut global_noise_data, &cell_unit_sphere_pos, ws, master_seed, octaves, frequency, attenuation, amplitude, start_idx_pure_noise, end_idx_pure_noise);
    simplex_noise::gen_height_adj_noise(&mut global_noise_data, hex_tile_count, lo, world_height, octaves, amplitude, attenuation, start_idx_height_adj_noise, end_idx_height_adj_noise, start_idx_pure_noise, end_idx_pure_noise);
    // println!("{:?}",cell_unit_sphere_pos);

    let mut neighbor_ids: Vec<i32> = vec![-1; (6 * hex_tile_count) as usize];
    let mut neighbor_cnt: Vec<u8> = vec![0; hex_tile_count as usize];
    helpers::gen_neighbor_id_and_cnt_data(&mut neighbor_ids, &mut neighbor_cnt, &cell_unit_sphere_pos, hex_tile_count, ws);
    // println!("{:?}",neighbor_ids);
    // println!("{:?}",neighbor_cnt);

    let mut hex_verts: Vec<f32> = vec![0.0; (7 * 3 * hex_tile_count) as usize];
    let mut norm_tan_height: Vec<f32> = vec![0.0; (7 * hex_tile_count) as usize];
    let mut tri_faces_idxs: Vec<u32> = vec![0; (6 * 3 * hex_tile_count) as usize];
    helpers::gen_hex_vert_and_norm_tan_and_tri_faces_data(&global_noise_data, &mut hex_verts, &mut norm_tan_height, &mut tri_faces_idxs, &cell_unit_sphere_pos, &neighbor_ids, &neighbor_cnt, hex_tile_count, ws, world_height);
    // println!("{:?}", hex_verts);
    // println!("{:?}", norm_tan_height);
    // println!("{:?}", tri_faces);

    let n = hex_tile_count;
    HexWorldPipelineRenderStruct { 
        ws,
        n,
        world_height,
        master_seed,
        start_idx_pure_noise,
        end_idx_pure_noise,
        start_idx_height_adj_noise,
        end_idx_height_adj_noise,
        cell_unit_sphere_pos,
        neighbor_ids,
        neighbor_cnt,
        hex_verts,
        norm_tan_height,
        tri_faces_idxs,
        global_noise_data,
    }
}

