use image::{DynamicImage, ImageBuffer, Rgb};
use kiss3d::prelude::*;
use kiss3d::resource::TextureManager;
use kiss3d::resource::vertex_index::VertexIndex;

use crate::worldgen_pipeline::HexWorldPipelineRenderStruct;

const HEIGHT_RAMP_TEX: &str = "hex_height_ramp";

#[derive(Default, Copy, Clone, PartialEq)]
pub enum RenderMode {
    #[default]
    PureSimplex,
    HexGlobe,
}

fn spawn_pure_simplex_view(scene: &mut SceneNode3d, pipeline: &HexWorldPipelineRenderStruct) {
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
        let base = curr_idx as usize * 3;
        let [x, y, z]: [f32; 3] = pipeline.cell_unit_sphere_pos[base..base + 3]
            .try_into()
            .expect("cell_unit_sphere_pos stride is xyz");
        let hex_val = pipeline.global_noise_data[curr_idx as usize];
        let radius = 1.0 + noise_effect_coeff * hex_val;

        scene
            .add_sphere(0.003)
            .set_position(Vec3::new(x * radius, y * radius, z * radius))
            .set_color(Color::new(1.0, 1.0, 1.0, 1.0));
    }
}

fn height_ramp_image() -> DynamicImage {
    let lo = [46u8, 140, 62];
    let hi = [12u8, 12, 12];
    DynamicImage::ImageRgb8(ImageBuffer::from_fn(256, 1, |x, _| {
        let t = x as f32 / 255.0;
        Rgb([
            (lo[0] as f32 + (hi[0] as f32 - lo[0] as f32) * t).round() as u8,
            (lo[1] as f32 + (hi[1] as f32 - lo[1] as f32) * t).round() as u8,
            (lo[2] as f32 + (hi[2] as f32 - lo[2] as f32) * t).round() as u8,
        ])
    }))
}

fn hex_globe_gpu_mesh(pipeline: &HexWorldPipelineRenderStruct) -> Rc<RefCell<GpuMesh3d>> {
    let verts = &pipeline.hex_verts;
    let norm_verts = pipeline.norm_tan_height.len();
    let mut coords = Vec::with_capacity(norm_verts);
    let mut uvs = Vec::with_capacity(norm_verts);
    for v in 0..norm_verts {
        let base = v * 3;
        coords.push(Vec3::new(verts[base], verts[base + 1], verts[base + 2]));
        let tan = pipeline.norm_tan_height[v];
        uvs.push(Vec2::new(tan, 0.5));
    }

    let idx = &pipeline.tri_faces_idxs;
    let mut faces = Vec::with_capacity(idx.len() / 3);
    for tri in idx.chunks_exact(3) {
        if tri[0] == tri[1] && tri[1] == tri[2] {
            continue;
        }
        faces.push([tri[0] as VertexIndex, tri[1] as VertexIndex, tri[2] as VertexIndex]);
    }

    Rc::new(RefCell::new(GpuMesh3d::new(
        coords,
        faces,
        None,
        Some(uvs),
        false,
    )))
}

fn spawn_hex_globe_view(scene: &mut SceneNode3d, camera: &mut OrbitCamera3d, pipeline: &HexWorldPipelineRenderStruct) {
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

    let radius = pipeline.ws as f32 * 0.4;
    *camera = OrbitCamera3d::new(Vec3::new(0.0, 0.0, 3.0 * radius), Vec3::ZERO);

    TextureManager::get_global_manager(|tm| {
        tm.add_image(height_ramp_image(), HEIGHT_RAMP_TEX);
    });

    scene
        .add_mesh(hex_globe_gpu_mesh(pipeline), Vec3::splat(radius))
        .set_color(Color::new(1.0, 1.0, 1.0, 1.0))
        .set_texture_with_name(HEIGHT_RAMP_TEX)
        .set_metallic(0.0)
        .set_roughness(0.92)
        .enable_backface_culling(true);
}

fn spawn_curr_mode(curr_mode: RenderMode, scene: &mut SceneNode3d, camera: &mut OrbitCamera3d, pipeline: &HexWorldPipelineRenderStruct) {
    match curr_mode {
        RenderMode::PureSimplex => spawn_pure_simplex_view(scene, pipeline),
        RenderMode::HexGlobe => spawn_hex_globe_view(scene, camera, pipeline),
    }
}

pub async fn render_hex_pipeline(pipeline: &HexWorldPipelineRenderStruct) {
    let mut window = Window::new("globe_render").await;
    window.set_ambient(0.25);
    let mut camera = OrbitCamera3d::new(Vec3::new(0.0, 0.0, 3.0), Vec3::ZERO);
    let mut scene = SceneNode3d::empty();

    let mut render_mode = RenderMode::HexGlobe;
    spawn_curr_mode(render_mode, &mut scene, &mut camera, pipeline);

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
                    scene = SceneNode3d::empty();
                    spawn_curr_mode(render_mode, &mut scene, &mut camera, pipeline);
                }

                _ => {}
            }
        }
    }
}
