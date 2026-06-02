use crate::clcg_seed_gen;

pub fn gen_influence_map(
    width: usize,
    height: usize,
    preset: SpottedInfluencePresetVals,
    world_master_seed: u64,
) -> Vec<Vec<f64>> {
    let preset = SpottedInfluencePreset::new(&preset);
    let SpottedInfluencePreset {
        k_blobs,
        blob_radius,
        low_max,
        p_peaks,
        peak_radius,
        peak_max,
    } = preset;

    let mut map = vec![vec![0.0; width]; height];
    let mut rng: clcg_seed_gen::Clcg = clcg_seed_gen::Clcg::new(world_master_seed);

    for _ in 0..k_blobs {
        let cx = rng.next_u32() % width as u32;
        let cy = rng.next_u32() % height as u32;
        stamp_disk(&mut map, cx as f64, cy as f64, blob_radius, low_max);
    }
    for _ in 0..p_peaks {
        let cx = rng.next_u32() % width as u32;
        let cy = rng.next_u32() % height as u32;
        stamp_disk(&mut map, cx as f64, cy as f64, peak_radius, peak_max);
    }
    map
}

fn stamp_disk(map: &mut [Vec<f64>], cx: f64, cy: f64, rad: f64, strength: f64) {
    let rad_i = rad.ceil() as i32;
    let cx_i = cx as i32;
    let cy_i = cy as i32;
    for dy in -rad_i..=rad_i {
        for dx in -rad_i..=rad_i {
            let dist = ((dx * dx + dy * dy) as f64).sqrt();
            if dist > rad {
                continue;
            }
            let t = 1.0 - dist / rad;
            let v = strength * t * t;
            let x_pos = cx_i + dx;
            let y_pos = cy_i + dy;
            if x_pos < 0
                || y_pos < 0
                || x_pos as usize >= map[0].len()
                || y_pos as usize >= map.len()
            {
                continue;
            }
            let x = (x_pos) as usize;
            let y = (y_pos) as usize;
            map[y][x] = map[y][x].max(v);
        }
    }
}

pub enum SpottedInfluencePresetVals {
    Low,
    Middle,
    High,
    VeryHigh,
}
pub struct SpottedInfluencePreset {
    k_blobs: usize,
    blob_radius: f64,
    low_max: f64,
    p_peaks: usize,
    peak_radius: f64,
    peak_max: f64,
}

impl SpottedInfluencePreset {

    pub fn new(preset: &SpottedInfluencePresetVals) -> Self {
        match preset {
            SpottedInfluencePresetVals::Low => Self {
                k_blobs: 1,
                blob_radius: 95.0,
                low_max: 20.0,
                p_peaks: 2,
                peak_radius: 15.0,
                peak_max: 100.0,
            },
            SpottedInfluencePresetVals::Middle => Self {
                k_blobs: 2,
                blob_radius: 115.0,
                low_max: 20.0,
                p_peaks: 2,
                peak_radius: 20.0,
                peak_max: 100.0,
            },
            SpottedInfluencePresetVals::High => Self {
                k_blobs: 3,
                blob_radius: 135.0,
                low_max: 20.0,
                p_peaks: 3,
                peak_radius: 25.0,
                peak_max: 100.0,
            },
            SpottedInfluencePresetVals::VeryHigh => Self {
                k_blobs: 4,
                blob_radius: 155.0,
                low_max: 20.0,
                p_peaks: 4,
                peak_radius: 30.0,
                peak_max: 100.0,
            },
        }
    }
}
