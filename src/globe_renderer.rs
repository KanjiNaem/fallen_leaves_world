use kiss3d::prelude::*;

use crate::{helpers, worldgen_pipeline::HexWorldPipelineStruct};

#[derive(Default, Copy, Clone, PartialEq)]
pub enum RenderMode {
    #[default]
    PureSimplex,
    HexGlobe,
}

fn spawn_pure_simplex_view(scene: &mut SceneNode3d, pipeline: &HexWorldPipelineStruct) {
    scene
        .add_light(
            Light::directional(Vec3::new(0.35, -1.0, 0.25))
                .with_intensity(10.0)
                .with_color(Color::new(1.0, 1.0, 1.0, 1.0)),
        )
        .set_position(Vec3::new(0.0, 3.0, 0.0));
    scene
        .add_light(Light::point(6.0).with_intensity(0.45))
        .set_position(Vec3::new(-2.0, 1.5, 2.5));

    let noise_effect_coeff = 0.25;
    for curr_idx in pipeline.start_idx_pure_noise..=pipeline.end_idx_pure_noise {
        let [x, y, z] = helpers::cell_pos_on_sphere(curr_idx, pipeline.ws);
        let hex_val = pipeline.global_data[curr_idx as usize];
        let radius = 1.0 + noise_effect_coeff * hex_val;

        scene
            .add_sphere(0.003)
            .set_position(Vec3::new(x * radius, y * radius, z * radius))
            .set_color(Color::new(1.0, 1.0, 1.0, 1.0));
    }
}

fn spawn_hex_globe_view(_scene: &mut SceneNode3d, _pipeline: &HexWorldPipelineStruct) {}

fn spawn_curr_mode(curr_mode: RenderMode, scene: &mut SceneNode3d, pipeline: &HexWorldPipelineStruct) {
    match curr_mode {
        RenderMode::PureSimplex => spawn_pure_simplex_view(scene, pipeline),
        RenderMode::HexGlobe => spawn_hex_globe_view(scene, pipeline),
    }
}

pub async fn render_hex_pipeline(pipeline: &HexWorldPipelineStruct) {
    let mut window = Window::new("globe_render").await;
    window.set_ambient(0.25);
    let mut camera = OrbitCamera3d::new(Vec3::new(0.0, 0.0, 3.0), Vec3::ZERO);
    let mut scene = SceneNode3d::empty();

    let mut render_mode = RenderMode::default();
    spawn_curr_mode(render_mode, &mut scene, pipeline);

    let switch_render_mode = |curr_mode: RenderMode| {
        if curr_mode == RenderMode::PureSimplex {
            RenderMode::HexGlobe
        } else {
            RenderMode::PureSimplex
        }
    };

    while window.render_3d(&mut scene, &mut camera).await {
        for event in window.events().iter() {
            match event.value {
                WindowEvent::Key(Key::G, Action::Press, _) => {
                    render_mode = switch_render_mode(render_mode);
                }

                _ => {}
            }
        }
    }
}
