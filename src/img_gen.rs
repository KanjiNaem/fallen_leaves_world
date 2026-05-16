use std::fs;
use std::path::Path;
use image::{ImageBuffer, Luma};


pub fn gen_greyscale_img_from_vec(noise_vec: Vec<Vec<f64>>) -> () {

    let width = noise_vec.len();
    let height = noise_vec[0].len();
    let noise_vec_flat: Vec<f64> = noise_vec.iter().flatten().cloned().collect();
    
    let mut min_val = noise_vec_flat[0];
    let mut max_val = noise_vec_flat[0];
    for &curr_val in &noise_vec_flat {
        if min_val > curr_val {
            min_val = curr_val;
        };
        if max_val < curr_val {
            max_val = curr_val
        }
    }

    let range = if (max_val - min_val).abs() < f64::EPSILON {
        1.0 // fix /0 issue, trust trust
    } else {
        max_val - min_val
    };

    let mut img: ImageBuffer<_, Vec<_>> = ImageBuffer::new(width as u32, height as u32);

    for y in 0..height {
        for x in 0..width {

            let val = noise_vec[x][y];
            let norm = (val - min_val) / range;
            let pixel_val = (norm * 255.0).round() as u8;
            img.put_pixel(x as u32, y as u32, Luma([pixel_val]));

        }
    }

    let out_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("output_imgs");
    fs::create_dir_all(&out_dir).unwrap();
    let out_path = out_dir.join("greyscale_perlin_noise_output.png");
    img.save(&out_path).unwrap();

    println!("saved to {}", out_path.display());
}