use rayon::prelude::*;

use crate::helpers;

pub fn gen_moisture_map(width: usize, height: usize, terrain_map: Vec<Vec<f64>>, water_lvl: f64) -> Vec<Vec<f64>> {

    let water_body_groups = helpers::group_water_bodies(width, height, &terrain_map, water_lvl);
    let water_body_group_sizes = helpers::water_body_group_size(water_body_groups);

    


    return vec![vec![0.0; 1]; 1];
}