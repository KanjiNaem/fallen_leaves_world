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

/// High percentile of positive rainfall on land — stable display scale when a few peaks spike.
fn land_rainfall_display_scale(
    rainfall: &Vec<Vec<f64>>,
    terrain: &Vec<Vec<f64>>,
    water_level: f64,
) -> f64 {
    let mut values: Vec<f64> = Vec::new();
    for y in 0..rainfall.len() {
        for x in 0..rainfall[0].len() {
            if terrain[y][x] <= water_level {
                continue;
            }
            let v = rainfall[y][x];
            if v > 0.0 {
                values.push(v);
            }
        }
    }
    if values.is_empty() {
        return 1.0;
    }
    values.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let idx = ((values.len() - 1) as f64 * 0.98).round() as usize;
    values[idx].max(1.0)
}

#[inline]
fn lerp_channel(a: f64, b: f64, t: f64) -> u8 {
    (a + (b - a) * t).round().clamp(0.0, 255.0) as u8
}

#[inline]
fn blend_rgb(base: Rgb<u8>, overlay: Rgb<u8>, overlay_weight: f64) -> Rgb<u8> {
    let t = overlay_weight.clamp(0.0, 1.0);
    Rgb([
        lerp_channel(base.0[0] as f64, overlay.0[0] as f64, t),
        lerp_channel(base.0[1] as f64, overlay.0[1] as f64, t),
        lerp_channel(base.0[2] as f64, overlay.0[2] as f64, t),
    ])
}

const OCEAN_RGB: Rgb<u8> = Rgb([20, 66, 114]);

#[inline]
fn terrain_elevation_color_with_water(
    terrain: &Vec<Vec<f64>>,
    x: usize,
    y: usize,
    min_val: f64,
    range: f64,
    norm_water_lvl: f64,
    land_denom: f64,
    land_palette: &LandElevationPalette,
) -> Rgb<u8> {
    let norm = ((terrain[y][x] - min_val) / range).clamp(0.0, 1.0);
    if norm < norm_water_lvl {
        OCEAN_RGB
    } else {
        let land_scale = if norm_water_lvl >= 1.0 {
            0.0
        } else {
            (norm - norm_water_lvl) / land_denom
        };
        land_palette.land_color(land_scale.clamp(0.0, 1.0))
    }
}

#[inline]
fn terrain_elevation_color(
    terrain: &Vec<Vec<f64>>,
    x: usize,
    y: usize,
    min_val: f64,
    range: f64,
    land_palette: &LandElevationPalette,
) -> Rgb<u8> {
    let norm = ((terrain[y][x] - min_val) / range).clamp(0.0, 1.0);
    land_palette.land_color(norm.clamp(0.0, 1.0))
}

#[derive(Clone, Debug)]
pub struct LandElevationPalette {
    pub t_yellow: f64,
    pub t_orange: f64,
    pub t_red: f64,
    pub green: [f64; 3],
    pub yellow: [f64; 3],
    pub orange: [f64; 3],
    pub red: [f64; 3],
    pub black: [f64; 3],
}

impl Default for LandElevationPalette {
    fn default() -> Self {
        Self {
            t_yellow: 0.28,
            t_orange: 0.52,
            t_red: 0.78,
            green: [34.0, 160.0, 55.0],
            yellow: [255.0, 230.0, 45.0],
            orange: [255.0, 130.0, 25.0],
            red: [200.0, 25.0, 25.0],
            black: [0.0, 0.0, 0.0],
        }
    }
}

impl LandElevationPalette {
    #[inline]
    pub fn land_color(&self, t: f64) -> Rgb<u8> {
        let t = t.clamp(0.0, 1.0);
        let ty = self.t_yellow.clamp(f64::EPSILON, 1.0 - 3.0 * f64::EPSILON);
        let to = self
            .t_orange
            .clamp(ty + f64::EPSILON, 1.0 - 2.0 * f64::EPSILON);
        let tr = self.t_red.clamp(to + f64::EPSILON, 1.0 - f64::EPSILON);

        let stops: [(f64, [f64; 3]); 5] = [
            (0.0, self.green),
            (ty, self.yellow),
            (to, self.orange),
            (tr, self.red),
            (1.0, self.black),
        ];

        let mut i = 0usize;
        while i + 1 < stops.len() && t > stops[i + 1].0 {
            i += 1;
        }
        let (t0, c0) = stops[i];
        let (t1, c1) = stops[i + 1];
        let span = (t1 - t0).max(1e-9);
        let u = ((t - t0) / span).clamp(0.0, 1.0);
        Rgb([
            lerp_channel(c0[0], c1[0], u),
            lerp_channel(c0[1], c1[1], u),
            lerp_channel(c0[2], c1[2], u),
        ])
    }
}

const FLOW_MOISTURE_OVERLAY_MIN: f64 = 0.42;
const FLOW_MOISTURE_OVERLAY_MAX: f64 = 0.78;

/// Tan → cyan → pale blue ramp used for flow-based local moisture (mountain rainfall).
fn flow_moisture_color(t: f64) -> Rgb<u8> {
    const DRY_LAND: Rgb<u8> = Rgb([168, 145, 98]);
    const WET_LOW: Rgb<u8> = Rgb([55, 145, 175]);
    const WET_HIGH: Rgb<u8> = Rgb([235, 245, 255]);
    let t = t.clamp(0.0, 1.0);
    if t < 0.5 {
        let u = t / 0.5;
        Rgb([
            lerp_channel(DRY_LAND.0[0] as f64, WET_LOW.0[0] as f64, u),
            lerp_channel(DRY_LAND.0[1] as f64, WET_LOW.0[1] as f64, u),
            lerp_channel(DRY_LAND.0[2] as f64, WET_LOW.0[2] as f64, u),
        ])
    } else {
        let u = (t - 0.5) / 0.5;
        Rgb([
            lerp_channel(WET_LOW.0[0] as f64, WET_HIGH.0[0] as f64, u),
            lerp_channel(WET_LOW.0[1] as f64, WET_HIGH.0[1] as f64, u),
            lerp_channel(WET_LOW.0[2] as f64, WET_HIGH.0[2] as f64, u),
        ])
    }
}

/// Moisture ramp: pale blue (dry) through saturated blues to red at `t == 1.0` (max moisture).
fn moisture_color(t: f64) -> Rgb<u8> {
    let t = t.clamp(0.0, 1.0);
    let stops: [(f64, [f64; 3]); 5] = [
        (0.0, [255.0, 255.0, 255.0]),
        (0.2, [175.0, 215.0, 250.0]),
        (0.45, [100.0, 170.0, 230.0]),
        (0.7, [45.0, 115.0, 195.0]),
        (1.0, [255.0, 0.0, 0.0]),
    ];

    let mut i = 0usize;
    while i + 1 < stops.len() && t > stops[i + 1].0 {
        i += 1;
    }
    let (t0, c0) = stops[i];
    let (t1, c1) = stops[i + 1];
    let span = (t1 - t0).max(1e-9);
    let u = ((t - t0) / span).clamp(0.0, 1.0);
    Rgb([
        lerp_channel(c0[0], c1[0], u),
        lerp_channel(c0[1], c1[1], u),
        lerp_channel(c0[2], c1[2], u),
    ])
}

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

pub fn gen_local_flow_rainfall_map_img(
    rainfall: &Vec<Vec<f64>>,
    terrain: &Vec<Vec<f64>>,
    water_level: f64,
    land_palette: &LandElevationPalette,
    file_name: String,
) {
    let grid_h = rainfall.len();
    let grid_w = rainfall[0].len();

    let (min_val, _, range) = min_max_range_2d(terrain);
    let norm_water_lvl = ((water_level - min_val) / range).clamp(0.0, 1.0);
    let land_denom = (1.0 - norm_water_lvl).max(f64::EPSILON);

    let rain_scale = land_rainfall_display_scale(rainfall, terrain, water_level);

    let mut img = RgbImage::new(grid_w as u32, grid_h as u32);

    for y in 0..grid_h {
        for x in 0..grid_w {
            let elevation_color = terrain_elevation_color_with_water(
                terrain,
                x,
                y,
                min_val,
                range,
                norm_water_lvl,
                land_denom,
                land_palette,
            );

            let px_color = if terrain[y][x] <= water_level {
                elevation_color
            } else {
                let t = (rainfall[y][x] / rain_scale).clamp(0.0, 1.0);
                let tint = flow_moisture_color(t);
                let overlay = FLOW_MOISTURE_OVERLAY_MIN
                    + t * (FLOW_MOISTURE_OVERLAY_MAX - FLOW_MOISTURE_OVERLAY_MIN);
                blend_rgb(elevation_color, tint, overlay)
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

/// Renders land moisture over terrain. `max_moisture` is cell capacity (0..=max);
/// color/overlay use `stored / max_moisture` so higher caps need more stored moisture to reach red.
pub fn gen_moisture_map_img(
    rainfall: &Vec<Vec<f64>>,
    terrain: &Vec<Vec<f64>>,
    water_level: f64,
    max_moisture: f64,
    land_palette: &LandElevationPalette,
    file_name: String,
) {
    let grid_h = rainfall.len();
    let grid_w = rainfall[0].len();

    let (min_val, _, range) = min_max_range_2d(terrain);
    let norm_water_lvl = ((water_level - min_val) / range).clamp(0.0, 1.0);
    let land_denom = (1.0 - norm_water_lvl).max(f64::EPSILON);

    let moisture_denom = if max_moisture > f64::EPSILON {
        max_moisture
    } else {
        1.0
    };

    let mut img = RgbImage::new(grid_w as u32, grid_h as u32);

    for y in 0..grid_h {
        for x in 0..grid_w {
            let elevation_color = terrain_elevation_color_with_water(
                terrain,
                x,
                y,
                min_val,
                range,
                norm_water_lvl,
                land_denom,
                land_palette,
            );

            let px_color = if terrain[y][x] <= water_level {
                elevation_color
            } else {
                let moisture_t = (rainfall[y][x] / moisture_denom).clamp(0.0, 1.0);
                if moisture_t <= 0.0 {
                    elevation_color
                } else {
                    let tint = moisture_color(moisture_t);
                    blend_rgb(elevation_color, tint, moisture_t)
                }
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

pub fn gen_perlin_rgb(
    noise_vec: &Vec<Vec<f64>>,
    land_palette: &LandElevationPalette,
    file_name: String,
) {
    let grid_h = noise_vec.len();
    let grid_w = noise_vec[0].len();
    let (min_val, _, range) = min_max_range_2d(noise_vec);
    let mut img = RgbImage::new(grid_w as u32, grid_h as u32);

    for y in 0..grid_h {
        for x in 0..grid_w {
            let px_color = terrain_elevation_color(
                noise_vec,
                x,
                y,
                min_val,
                range,
                land_palette,
            );
            img.put_pixel(x as u32, y as u32, px_color);
        }
    }

    let out_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("output_imgs");
    fs::create_dir_all(&out_dir).unwrap();
    let out_path = out_dir.join(file_name);
    img.save(&out_path).unwrap();

    println!("saved to {}", out_path.display());
}

pub fn gen_perlin_rgb_with_water(
    noise_vec: &Vec<Vec<f64>>,
    water_level: f64,
    land_palette: &LandElevationPalette,
    file_name: String,
) {
    let grid_h = noise_vec.len();
    let grid_w = noise_vec[0].len();
    let (min_val, _, range) = min_max_range_2d(noise_vec);

    let norm_water_lvl = ((water_level - min_val) / range).clamp(0.0, 1.0);
    let land_denom = (1.0 - norm_water_lvl).max(f64::EPSILON);

    let mut img = RgbImage::new(grid_w as u32, grid_h as u32);

    for y in 0..grid_h {
        for x in 0..grid_w {
            let px_color = terrain_elevation_color_with_water(
                noise_vec,
                x,
                y,
                min_val,
                range,
                norm_water_lvl,
                land_denom,
                land_palette,
            );
            img.put_pixel(x as u32, y as u32, px_color);
        }
    }

    let out_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("output_imgs");
    fs::create_dir_all(&out_dir).unwrap();
    let out_path = out_dir.join(file_name);
    img.save(&out_path).unwrap();

    println!("saved to {}", out_path.display());
}

fn put_square(img: &mut RgbImage, cx: i32, cy: i32, half: i32, color: Rgb<u8>) {
    let width = img.width() as i32;
    let height = img.height() as i32;
    for dy in -half..=half {
        for dx in -half..=half {
            let x = cx + dx;
            let y = cy + dy;
            if x >= 0 && y >= 0 && x < width && y < height {
                img.put_pixel(x as u32, y as u32, color);
            }
        }
    }
}

fn draw_thick_line_rgb(
    img: &mut RgbImage,
    x0: i32,
    y0: i32,
    x1: i32,
    y1: i32,
    stroke: i32,
    color: Rgb<u8>,
) {
    let mut x = x0;
    let mut y = y0;
    let dx = (x1 - x0).abs();
    let dy = -(y1 - y0).abs();
    let sx = if x0 < x1 { 1 } else { -1 };
    let sy = if y0 < y1 { 1 } else { -1 };
    let mut err = dx + dy;

    loop {
        put_square(img, x, y, stroke, color);
        if x == x1 && y == y1 {
            break;
        }
        let e2 = 2 * err;
        if e2 >= dy {
            err += dy;
            x += sx;
        }
        if e2 <= dx {
            err += dx;
            y += sy;
        }
    }
}

fn draw_flow_arrow(
    img: &mut RgbImage,
    x: i32,
    y: i32,
    ux: i32,
    uy: i32,
    shaft_len: i32,
    stroke: i32,
    head_len: i32,
    color: Rgb<u8>,
) {
    if ux == 0 && uy == 0 {
        return;
    }
    let tip_x = x + ux * shaft_len;
    let tip_y = y + uy * shaft_len;
    draw_thick_line_rgb(img, x, y, tip_x, tip_y, stroke, color);

    let px = -uy;
    let py = ux;
    let back_x = tip_x - ux * head_len;
    let back_y = tip_y - uy * head_len;
    let wing = head_len;
    draw_thick_line_rgb(
        img,
        tip_x,
        tip_y,
        back_x + px * wing,
        back_y + py * wing,
        stroke,
        color,
    );
    draw_thick_line_rgb(
        img,
        tip_x,
        tip_y,
        back_x - px * wing,
        back_y - py * wing,
        stroke,
        color,
    );
}

pub fn gen_upwind_sparse_arrow_img(
    upwind: &Vec<Vec<Option<(usize, usize)>>>,
    terrain: &Vec<Vec<f64>>,
    water_level: f64,
    land_palette: &LandElevationPalette,
    step: usize,
    file_name: String,
) {
    let grid_h = terrain.len();
    let grid_w = terrain[0].len();
    let step = step.max(1);
    let (min_val, _, range) = min_max_range_2d(terrain);
    let norm_water_lvl = ((water_level - min_val) / range).clamp(0.0, 1.0);
    let land_denom = (1.0 - norm_water_lvl).max(f64::EPSILON);
    let arrow = Rgb([120u8, 235, 255]);

    let mut img = RgbImage::new(grid_w as u32, grid_h as u32);

    for y in 0..grid_h {
        for x in 0..grid_w {
            let px_color = terrain_elevation_color_with_water(
                terrain,
                x,
                y,
                min_val,
                range,
                norm_water_lvl,
                land_denom,
                land_palette,
            );
            img.put_pixel(x as u32, y as u32, px_color);
        }
    }

    for y in (0..grid_h).step_by(step) {
        for x in (0..grid_w).step_by(step) {
            if terrain[y][x] <= water_level {
                continue;
            }
            let Some((nx, ny)) = upwind[y][x] else {
                continue;
            };
            // parent is upwind; moisture flows downwind (+grad Φ)
            let ux = x as i32 - nx as i32;
            let uy = y as i32 - ny as i32;
            const SHAFT_LEN: i32 = 18;
            const STROKE: i32 = 2;
            const HEAD_LEN: i32 = 6;
            draw_flow_arrow(
                &mut img, x as i32, y as i32, ux, uy, SHAFT_LEN, STROKE, HEAD_LEN, arrow,
            );
        }
    }

    let out_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("output_imgs");
    fs::create_dir_all(&out_dir).unwrap();
    let out_path = out_dir.join(file_name);
    img.save(&out_path).unwrap();
    println!("saved to {}", out_path.display());
}
