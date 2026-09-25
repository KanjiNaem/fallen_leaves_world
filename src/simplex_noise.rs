use core::panic;
use crate::{clcg_seed_gen::Clcg, helpers};

const SKEW_3D: f32 = 1.0 / 3.0;
const UNSKEW_3D: f32 = 1.0 / 6.0;
const GRAD_3D_SIMPLEX: [f32; 36] = [1.0, 1.0, 0.0, -1.0, 1.0, 0.0, 1.0, -1.0, 0.0, -1.0, -1.0, 0.0, 1.0, 0.0, 1.0, -1.0, 0.0, 1.0, 1.0, 0.0, -1.0, -1.0, 0.0, -1.0, 0.0, 1.0, 1.0, 0.0, -1.0, 1.0, 0.0, 1.0, -1.0, 0.0, -1.0, -1.0]; // gradients indexed via idx * 3

// runs on data[0..(10 * ws + 2)]
pub fn gen_octaved_simplex(data: &mut Vec<f32>, ws: i32, master_seed: u64, octaves: i32, mut frequency: f32, attenuation: f32, mut amplitude: f32, data_start_idx: i32, data_end_idx: i32) -> () {
    let n_cells = 10 * ws * ws + 2;
    let mut rng_from_seed = Clcg::new(master_seed);

    for _ in 0..octaves {
        for idx in 0..n_cells {
            let cell_sphere_pos = helpers::cell_pos_on_sphere(idx, ws);
            let data_entry_pos = data_start_idx + idx;
            if data_entry_pos > data_end_idx {
                // not sure aboout using a panic here, however if this fucks up everything is fucked up either way so might as well kill it here!
                panic!("gen_octaved_simplex tried to enter data at pos: {}, when it should only enter data until pos: {}", data_entry_pos, data_end_idx);
            }

            data[data_entry_pos as usize] += amplitude * noise_step_3d(cell_sphere_pos[0] * frequency, cell_sphere_pos[1] * frequency, cell_sphere_pos[2] * frequency, rng_from_seed.next_u64()) 
        }
        frequency *= 2.0;
        amplitude *= attenuation;
    }
    ()
}

pub fn gen_height_adj_noise(data: &mut Vec<f32>, hex_tile_count: i32, lo: f32, hi: f32, octaves: i32, amplitude: f32, attenuation: f32, data_start_idx: i32, data_end_idx: i32, start_idx_pure_noise: i32, end_idx_pure_noise: i32) -> () {
    let max_amp: f32 = (0..octaves).map(|curr| amplitude * attenuation.powi(curr)).sum();
    for curr_pure_noise_val in start_idx_pure_noise..=end_idx_pure_noise {
        let data_entry_pos = curr_pure_noise_val + hex_tile_count;
        if data_entry_pos > data_end_idx || data_entry_pos < data_start_idx {
            panic!("gen_height_adj_noise tried to enter data at pos: {}, when it should only enter data until pos: {}", data_entry_pos, data_end_idx);
        }

        let temp = (data[curr_pure_noise_val as usize] / max_amp).clamp(-1.0, 1.0);
        data[data_entry_pos as usize] = lo + (temp * 0.5 + 0.5) * (hi - lo);
    }
    ()
}

fn noise_step_3d(x_in: f32, y_in: f32, z_in: f32, master_seed: u64) -> f32 {

    // skew coord step
    let skew_fac = (x_in + y_in + z_in) * SKEW_3D;
    let x_sfloor = (x_in + skew_fac).floor();
    let y_sfloor = (y_in + skew_fac).floor();
    let z_sfloor = (z_in + skew_fac).floor();
    let unskewed_cell_orig = (x_sfloor + y_sfloor + z_sfloor) * UNSKEW_3D;
    let unskew_org_x0 = x_sfloor - unskewed_cell_orig;
    let unskew_org_y0 = y_sfloor - unskewed_cell_orig;
    let unskew_org_z0 = z_sfloor - unskewed_cell_orig;
    let x0 = x_in - unskew_org_x0;
    let y0 = y_in - unskew_org_y0;
    let z0 = z_in - unskew_org_z0;

    // simplical division
    let mut i1: f32 = 0.0; // first corner simplex offset
    let mut j1: f32 = 0.0;
    let mut k1: f32 = 0.0;
    let mut i2: f32 = 0.0; // second corner simplex offset
    let mut j2: f32 = 0.0;
    let mut k2: f32 = 0.0;

    if x0 >= y0 {
        if y0 >= z0 { 
            i1 = 1.0; // X Y Z
            i2 = 1.0;
            j2 = 1.0;            
        } else if x0 >= z0 { 
            i1 = 1.0; // X Z Y
            i2 = 1.0;
            k2 = 1.0;
        } else {    
            k1 = 1.0; // Z X Y
            i2 = 1.0;
            k2 = 1.0;
        }
    }

    if x0 < y0 {
        if y0 < z0 {
            k1 = 1.0; // Z Y X
            j2 = 1.0;
            k2 = 1.0;
        } else if x0 < z0 {
            j1 = 1.0; // Y Z X
            j2 = 1.0;
            k2 = 1.0;
        } else {
            j1 = 1.0; // Y X Z
            i2 = 1.0;
            j2 = 1.0;
        }
    }

    // second corner offset in xyz
    let x1:f32 = x0 - i1 + UNSKEW_3D;
    let y1:f32 = y0 - j1 + UNSKEW_3D;
    let z1:f32 = z0 - k1 + UNSKEW_3D;
    
    // third corner offset in xyz
    let x2:f32 = x0 - i2 + 2.0 * UNSKEW_3D;
    let y2:f32 = y0 - j2 + 2.0 * UNSKEW_3D;
    let z2:f32 = z0 - k2 + 2.0 * UNSKEW_3D;
    
    // fourth corner offset in xyz 
    let x3:f32 = x0 - 1.0 + 3.0 * UNSKEW_3D;
    let y3:f32 = y0 - 1.0 + 3.0 * UNSKEW_3D;
    let z3:f32 = z0 - 1.0 + 3.0 * UNSKEW_3D;

    // grad selection 
    let gradient_hash_0_set = funny_hash(x_sfloor, y_sfloor, z_sfloor, master_seed);
    let gradient_hash_1_set = funny_hash(x_sfloor + i1, y_sfloor + j1, z_sfloor + k1, master_seed);
    let gradient_hash_2_set = funny_hash(x_sfloor + i2, y_sfloor + j2, z_sfloor + k2, master_seed);
    let gradient_hash_3_set = funny_hash(x_sfloor + 1.0, y_sfloor + 1.0, z_sfloor + 1.0, master_seed); 

    // kernel summation
    let mut unskewed_0: f32 = 0.6 - x0 * x0 - y0 * y0 - z0 * z0;
    let mut unskewed_1: f32 = 0.6 - x1 * x1 - y1 * y1 - z1 * z1;
    let mut unskewed_2: f32 = 0.6 - x2 * x2 - y2 * y2 - z2 * z2;
    let mut unskewed_3: f32 = 0.6 - x3 * x3 - y3 * y3 - z3 * z3;
    let mut noise_contrib_0 = 0.0;
    let mut noise_contrib_1 = 0.0;
    let mut noise_contrib_2 = 0.0;
    let mut noise_contrib_3 = 0.0;
    if unskewed_0 >= 0.0 {
        unskewed_0 *= unskewed_0;
        noise_contrib_0 = unskewed_0 * unskewed_0 * dot_3d(GRAD_3D_SIMPLEX[gradient_hash_0_set], GRAD_3D_SIMPLEX[gradient_hash_0_set + 1], GRAD_3D_SIMPLEX[gradient_hash_0_set + 2], x0, y0, z0)
    }
    
    if unskewed_1 >= 0.0 {
        unskewed_1 *= unskewed_1;
        noise_contrib_1 = unskewed_1 * unskewed_1 * dot_3d(GRAD_3D_SIMPLEX[gradient_hash_1_set], GRAD_3D_SIMPLEX[gradient_hash_1_set + 1], GRAD_3D_SIMPLEX[gradient_hash_1_set + 2], x1, y1, z1);
    }

    if unskewed_2 >= 0.0 {
        unskewed_2 *= unskewed_2;
        noise_contrib_2 = unskewed_2 * unskewed_2 * dot_3d(GRAD_3D_SIMPLEX[gradient_hash_2_set], GRAD_3D_SIMPLEX[gradient_hash_2_set + 1], GRAD_3D_SIMPLEX[gradient_hash_2_set + 2], x2, y2, z2)
    }

    if unskewed_3 >= 0.0 {
        unskewed_3 *= unskewed_3;
        noise_contrib_3 = unskewed_3 * unskewed_3 * dot_3d(GRAD_3D_SIMPLEX[gradient_hash_3_set], GRAD_3D_SIMPLEX[gradient_hash_3_set + 1], GRAD_3D_SIMPLEX[gradient_hash_3_set + 2], x3, y3, z3)
    }

    32.0 * (noise_contrib_0 + noise_contrib_1 + noise_contrib_2 + noise_contrib_3)
}

fn funny_hash(x: f32, y: f32, z: f32, seed: u64) -> usize {
    let mut funny: i32 = (x * 374761393.0 + y * 668265263.0 + z * 1446898217.0) as i32;
    funny ^= seed as i32;
    funny ^= (seed >> 32) as i32;
    funny = (funny ^ (funny >> 13)).wrapping_mul(1274126177);
    funny ^= funny >> 16;
    ((funny as u32 as usize) % 12) * 3
}

fn dot_3d(x0: f32, y0: f32, z0: f32, x1: f32, y1: f32, z1: f32) -> f32 {
    x0 * x1 + y0 * y1 + z0 * z1 
}