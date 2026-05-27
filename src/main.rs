use fallen_leaves_world::{img_gen, world_gen_pipeline};

fn main() {
    let water_lvl = 120.0;
    let max_moisture = 1000.0;
    let pipeline = world_gen_pipeline::gen_world_pipeline_step_struct(
        1000,
        1000,
        500.0,
        500,
        8,
        0.7,
        water_lvl,
        max_moisture,
    );

    img_gen::gen_greyscale_img_from_vec(&pipeline.noise_base, format!("grey_pre_water_pass.png"));
    img_gen::gen_perlin_rgb_with_water(
        &pipeline.noise_base,
        water_lvl,
        &img_gen::LandElevationPalette::default(),
        format!("normal.png"),
    );
    img_gen::gen_perlin_rgb_with_water(
        &pipeline.smooth_noise,
        water_lvl,
        &img_gen::LandElevationPalette::default(),
        format!("smooth.png"),
    );
    img_gen::gen_perlin_rgb_with_water(
        &pipeline.wind_column_noise_base,
        0.0,
        &img_gen::LandElevationPalette::default(),
        format!("wind_col_base.png"),
    );
    img_gen::gen_upwind_sparse_arrow_img(
        &pipeline.wind_column_gradient,
        &pipeline.smooth_noise,
        water_lvl,
        &img_gen::LandElevationPalette::default(),
        24,
        format!("local_wind_upwind_arrows.png"),
    );
}
