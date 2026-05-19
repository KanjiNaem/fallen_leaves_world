use fallen_leaves_world::{img_gen, world_gen_pipeline};

fn main() {
    let water_lvl = 120.0;
    let rain_coeff = 1.0;
    let pipeline = world_gen_pipeline::gen_world_pipeline_step_struct(
        1000, 1000, 500, 8, 0.7, water_lvl, rain_coeff,
    );

    img_gen::gen_greyscale_img_from_vec(&pipeline.noise_base, format!("grey_pre_water_pass.png"));
    img_gen::gen_grey_with_waterlvl_highlighted(
        &pipeline.noise_base,
        water_lvl,
        &img_gen::LandElevationPalette::default(),
        format!("normal.png"),
    );
    img_gen::gen_grey_with_waterlvl_highlighted(
        &pipeline.smooth_noise,
        water_lvl,
        &img_gen::LandElevationPalette::default(),
        format!("smooth.png"),
    );
    img_gen::gen_rainfall_map_img(
        &pipeline.moisture_map,
        &pipeline.smooth_noise,
        water_lvl,
        format!("rainfall.png"),
    );
    img_gen::gen_grey_with_waterlvl_highlighted(&pipeline.wind_col_map,
        0.0,
        &img_gen::LandElevationPalette::default(),
        format!("wind_col_map.png"));
}
