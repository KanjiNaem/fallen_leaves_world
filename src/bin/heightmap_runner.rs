use fallen_leaves_world::{spotted_influence, temperature_map};
#[allow(unused_imports)]
use fallen_leaves_world::{band_influence, heightmap_view, img_gen, world_gen_pipeline};

#[kiss3d::main]
async fn main() {
    let water_lvl = 220.0;
    // higher max means proportionally more moisture needed to be considered very moist
    let max_moisture = 400.0;
    let band_magic_preset = band_influence::BandInfluencePresetVals::High;
    let spotted_chaos_preset = spotted_influence::SpottedInfluencePresetVals::High;
    let temp_preset = temperature_map::TempPresetVals::Middle;
    let temp_band_noise = band_influence::BandInfluencePresetVals::VeryHighNoDisks;
    let world_master_seed = 3345;

    // assume square maps only, panic otherwise for now
    let pipeline = world_gen_pipeline::gen_world_pipeline_step_struct(
        2000,
        2000,
        500.0,
        500,
        8,
        0.7,
        water_lvl,
        max_moisture,
        band_magic_preset,
        spotted_chaos_preset,
        temp_band_noise,
        temp_preset,
        world_master_seed,
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

    img_gen::gen_local_flow_rainfall_map_img(
        &pipeline.rainfall_map,
        &pipeline.smooth_noise,
        water_lvl,
        &img_gen::LandElevationPalette::default(),
        format!("local_rainfall_map.png"),
    );

    img_gen::gen_moisture_map_img(
        &pipeline.moisture_map,
        &pipeline.smooth_noise,
        water_lvl,
        max_moisture,
        &img_gen::LandElevationPalette::default(),
        format!("moisture_map.png"),
    );

    img_gen::gen_temperature_map_img(
        &pipeline.temperature_map,
        temperature_map::TEMP_DISPLAY_MIN,
        temperature_map::TEMP_DISPLAY_MAX,
        format!("temperature_map.png"),
    );

    img_gen::gen_perlin_rgb(
        &pipeline.magic_influence_map,
        &img_gen::LandElevationPalette::default(),
        format! {"magic_influence_map.png"},
    );

    img_gen::gen_perlin_rgb(
        &pipeline.chaos_influence_map,
        &img_gen::LandElevationPalette::default(),
        format! {"chaos_influence_map.png"},
    );

    // heightmap_view::view_heightmap(
    //     &pipeline.smooth_noise,
    //     water_lvl,
    //     true,
    //     heightmap_view::HeightmapViewConfig::default(),
    // )
    // .await;

    // heightmap_view::view_heightmap(
    //     &pipeline.chaos_influence_map,
    //     water_lvl,
    //     false,
    //     heightmap_view::HeightmapViewConfig::default(),
    // )
    // .await;
}
