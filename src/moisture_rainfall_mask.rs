use rayon::prelude::*;

pub fn gen_moisture_rainfall_mask(
    noise_map: &Vec<Vec<f64>>,
    water_lvl: f64,
    rain_coeff: f64,
) -> Vec<Vec<f64>> {
    let height = noise_map.len();
    let width = noise_map[0].len();
    let base_moisture: f64 = 70.0;
    let moisture_floor: f64 = 0.0;
    let moisture_ceiling: f64 = 120.0;
    let mut moisture_rainfall_mask: Vec<Vec<f64>> = vec![vec![0.0; width]; height];

    // default west to east air columns (row by row ez parallelize)
    // cloud moisture is internal; row stores rainfall at each cell (not moisture)
    // rainfall draws from moisture; only part of rainfall is lost from the cloud

    moisture_rainfall_mask
        .par_iter_mut()
        .enumerate()
        .for_each(|(y, row)| {
            let mut curr_cloud_moisture: f64 = 0.0;
            let mut prev_height = 1.0;
            let mut water_run: usize = 0;

            for x in 0..width {
                let curr_height = noise_map[y][x];
                let on_land = curr_height > water_lvl;

                if x == 0 {
                    curr_cloud_moisture = if on_land {
                        base_moisture * 0.3
                    } else {
                        base_moisture
                    };
                    water_run = if on_land { 0 } else { 1 };
                    prev_height = curr_height.clamp(1.0, 255.0);
                    row[x] = 0.0;
                    continue;
                }

                let prev_was_water = noise_map[y][x - 1] <= water_lvl;
                let prev_on_land = !prev_was_water;

                if !on_land {
                    water_run += 1;
                    curr_cloud_moisture =
                        (curr_cloud_moisture + 1.8).clamp(moisture_floor, moisture_ceiling);
                } else {
                    if prev_was_water {
                        let fetch = water_run.min(120) as f64 / 120.0;
                        let coastal_moisture = base_moisture * (0.55 + fetch * 0.45);
                        curr_cloud_moisture = curr_cloud_moisture.max(coastal_moisture);
                    }
                    water_run = 0;
                }

                let uplift = if on_land && prev_on_land {
                    (curr_height - prev_height).max(0.0)
                } else {
                    0.0
                };
                prev_height = curr_height.clamp(1.0, 255.0);

                let rainfall = if on_land && curr_cloud_moisture > 0.0 {
                    let orographic = if uplift > 0.1 {
                        let lift = 1.0 + uplift * 0.35 + uplift * uplift * 0.08;
                        curr_cloud_moisture * (uplift / 4.0) * lift * rain_coeff
                    } else {
                        0.0
                    };
                    let steady = curr_cloud_moisture * 0.03 * rain_coeff;
                    (orographic + steady).clamp(0.0, 100.0)
                } else {
                    0.0
                };

                curr_cloud_moisture =
                    (curr_cloud_moisture - rainfall * 0.16).clamp(moisture_floor, moisture_ceiling);

                if on_land {
                    curr_cloud_moisture =
                        (curr_cloud_moisture - 0.01).clamp(moisture_floor, moisture_ceiling);
                }

                row[x] = rainfall;
            }
        });

    return moisture_rainfall_mask;
}
