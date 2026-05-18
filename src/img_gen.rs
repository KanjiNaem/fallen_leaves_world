use image::{ImageBuffer, Luma, Rgb, RgbImage};
use std::fs;
use std::path::Path;

fn min_max_range_2d(noise_vec: &Vec<Vec<f64>>) -> (f64, f64, f64) {
    let noise_vec_flat: Vec<f64> = noise_vec.iter().flatten().cloned().collect();
    let mut min_val = noise_vec_flat[0];
    let mut max_val = noise_vec_flat[0];
    for &curr_val in &noise_vec_flat {
        if min_val > curr_val {
            min_val = curr_val;
        }
        if max_val < curr_val {
            max_val = curr_val;
        }
    }
    let range = if (max_val - min_val).abs() < f64::EPSILON {
        1.0
    } else {
        max_val - min_val
    };
    (min_val, max_val, range)
}

#[inline]
fn lerp_channel(a: f64, b: f64, t: f64) -> u8 {
    (a + (b - a) * t).round().clamp(0.0, 255.0) as u8
}

fn land_elevation_color(t: f64) -> Rgb<u8> {
    let t = t.clamp(0.0, 1.0);
    const STOPS: &[(f64, [f64; 3])] = &[
        (0.0, [28.0, 88.0, 42.0]),
        (0.22, [52.0, 128.0, 62.0]),
        (0.42, [168.0, 152.0, 98.0]),
        (0.65, [118.0, 102.0, 86.0]),
        (1.0, [248.0, 250.0, 255.0]),
    ];
    let mut i = 0usize;
    while i + 1 < STOPS.len() && t > STOPS[i + 1].0 {
        i += 1;
    }
    let (t0, c0) = STOPS[i];
    let (t1, c1) = STOPS[i + 1];
    let span = (t1 - t0).max(1e-9);
    let u = ((t - t0) / span).clamp(0.0, 1.0);
    Rgb([
        lerp_channel(c0[0], c1[0], u),
        lerp_channel(c0[1], c1[1], u),
        lerp_channel(c0[2], c1[2], u),
    ])
}

/// Full heightmap as greyscale (Luma). Uses grid min/max to map values to 0–255.
pub fn gen_greyscale_img_from_vec(noise_vec: &Vec<Vec<f64>>, file_name: String) {
    let grid_h = noise_vec.len();
    let grid_w = noise_vec[0].len();
    let (min_val, _max_val, range) = min_max_range_2d(noise_vec);

    let mut img: ImageBuffer<_, Vec<_>> = ImageBuffer::new(grid_w as u32, grid_h as u32);

    for y in 0..grid_h {
        for x in 0..grid_w {
            let val = noise_vec[y][x];
            let norm = ((val - min_val) / range).clamp(0.0, 1.0);
            let pixel_val = (norm * 255.0).round() as u8;
            img.put_pixel(x as u32, y as u32, Luma([pixel_val]));
        }
    }

    let out_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("output_imgs");
    fs::create_dir_all(&out_dir).unwrap();
    let out_path = out_dir.join(file_name);
    img.save(&out_path).unwrap();

    println!("saved to {}", out_path.display());
}

pub fn gen_grey_with_waterlvl_highlighted(noise_vec: &Vec<Vec<f64>>, water_level: f64, file_name: String) {
    let grid_h = noise_vec.len();
    let grid_w = noise_vec[0].len();
    let (min_val, _, range) = min_max_range_2d(noise_vec);

    let norm_water_lvl = ((water_level - min_val) / range).clamp(0.0, 1.0);
    let land_denom = (1.0 - norm_water_lvl).max(f64::EPSILON);

    let mut img = RgbImage::new(grid_w as u32, grid_h as u32);

    for y in 0..grid_h {
        for x in 0..grid_w {
            let norm = ((noise_vec[y][x] - min_val) / range).clamp(0.0, 1.0);

            let px_color = if norm < norm_water_lvl {
                Rgb([20, 66, 114])
            } else {
                let land_scale = if norm_water_lvl >= 1.0 {
                    0.0
                } else {
                    (norm - norm_water_lvl) / land_denom
                };
                let t = land_scale.clamp(0.0, 1.0);
                land_elevation_color(t)
            };

            img.put_pixel(x as u32, y as u32, px_color);
        }
    }

    let out_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("output_imgs");
    fs::create_dir_all(&out_dir).unwrap();
    let out_path = out_dir.join(file_name);
    img.save(&out_path).unwrap();

    println!("saved to {}", out_path.display());
}
