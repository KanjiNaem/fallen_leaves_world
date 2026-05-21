use fallen_leaves_world::{heightmap_view, world_gen_pipeline, img_gen};

#[kiss3d::main]
async fn main() {
    let water_lvl = 220.0;
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
    
    img_gen::gen_grey_with_waterlvl_highlighted(&pipeline.wind_column_noise_base,
        0.0,
        &img_gen::LandElevationPalette::default(),
        format!("wind_col_base.png"));
    
    img_gen::gen_upwind_sparse_arrow_img(
        &pipeline.wind_column_gradient,
        &pipeline.smooth_noise,
        water_lvl,
        &img_gen::LandElevationPalette::default(),
        24,
        format!("wind_upwind_arrows.png"),
    );
    
    img_gen::gen_rainfall_map_img(
        &pipeline.moisture_map,
        &pipeline.smooth_noise,
        water_lvl,
        &img_gen::LandElevationPalette::default(),
        format!("moisture_map.png"),
    );

    heightmap_view::view_heightmap(
        &pipeline.smooth_noise,
        water_lvl,
        true,
        heightmap_view::HeightmapViewConfig::default(),
    )
    .await;
}
