use fallen_leaves_world::{heightmap_view, world_gen_pipeline};

#[kiss3d::main]
async fn main() {
    let water_lvl = 120.0;
    let rain_coeff = 1.0;
    let pipeline = world_gen_pipeline::gen_world_pipeline_step_struct(
        1800, 1800, 500, 8, 0.7, water_lvl, rain_coeff,
    );

    heightmap_view::view_heightmap(
        &pipeline.smooth_noise,
        water_lvl,
        true,
        heightmap_view::HeightmapViewConfig::default(),
    )
    .await;
}
