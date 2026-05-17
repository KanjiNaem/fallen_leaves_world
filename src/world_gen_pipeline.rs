use crate::perlin_greyscale;

// elevation noise --> island mask

pub fn gen_world(width: usize, height: usize, start_period: usize, octaves: usize, attenuation: f64) -> () {
    let elevation_noise = perlin_greyscale::gen_octaved_perlin_greyscale(width, height, start_period, octaves, attenuation);


    
}