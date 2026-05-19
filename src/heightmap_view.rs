//! Interactive 3D heightmap preview (kiss3d).

use crate::img_gen::LandElevationPalette;
use image::{ImageBuffer, Rgb};
use kiss3d::prelude::*;
use kiss3d::resource::vertex_index::VertexIndex;
use std::cell::RefCell;
use std::io::Cursor;
use std::rc::Rc;

/// Downsample step and world scaling for the preview mesh.
pub struct HeightmapViewConfig {
    pub step: usize,
    pub xy_scale: f32,
    pub z_scale: f32,
    /// Pull the water surface down by this fraction of map height span (avoids z-fighting at shores).
    pub water_depth_bias_frac: f32,
}

impl Default for HeightmapViewConfig {
    fn default() -> Self {
        Self {
            step: 5,
            xy_scale: 1.0,
            z_scale: 0.1,
            water_depth_bias_frac: 0.012,
        }
    }
}

pub struct HeightmapMesh {
    pub vertices: Vec<Vec3>,
    pub faces: Vec<[VertexIndex; 3]>,
    pub uvs: Vec<Vec2>,
}

fn grid_min_max(grid: &[Vec<f64>]) -> (f64, f64, f64) {
    let mut min_val = grid[0][0];
    let mut max_val = grid[0][0];
    for row in grid {
        for &v in row {
            min_val = min_val.min(v);
            max_val = max_val.max(v);
        }
    }
    let range = if (max_val - min_val).abs() < f64::EPSILON {
        1.0
    } else {
        max_val - min_val
    };
    (min_val, max_val, range)
}

fn downsample_dims(grid_len: usize, step: usize) -> usize {
    let step = step.max(1);
    (grid_len - 1) / step + 1
}

/// RGB height colormap as PNG bytes (same ocean / land palette as 2D export).
pub fn height_colormap_png(grid: &[Vec<f64>], step: usize, water_level: f64) -> Vec<u8> {
    let step = step.max(1);
    let mesh_h = downsample_dims(grid.len(), step);
    let mesh_w = downsample_dims(grid[0].len(), step);
    let (min_val, _, range) = grid_min_max(grid);
    let norm_water = ((water_level - min_val) / range).clamp(0.0, 1.0);
    let land_denom = (1.0 - norm_water).max(f64::EPSILON);
    let palette = LandElevationPalette::default();
    let ocean = Rgb([20u8, 66, 114]);

    let mut img: ImageBuffer<Rgb<u8>, Vec<u8>> = ImageBuffer::new(mesh_w as u32, mesh_h as u32);

    for yi in 0..mesh_h {
        let sy = (yi * step).min(grid.len() - 1);
        for xi in 0..mesh_w {
            let sx = (xi * step).min(grid[0].len() - 1);
            let norm = ((grid[sy][sx] - min_val) / range).clamp(0.0, 1.0);
            let rgb = if norm < norm_water {
                ocean
            } else {
                let land_t = ((norm - norm_water) / land_denom).clamp(0.0, 1.0);
                palette.land_color(land_t)
            };
            img.put_pixel(xi as u32, yi as u32, rgb);
        }
    }

    let mut png = Vec::new();
    img.write_to(
        &mut Cursor::new(&mut png),
        image::ImageFormat::Png,
    )
    .expect("encode height colormap png");
    png
}

/// World Y from a heightmap sample (zero at grid minimum, same basis as the colormap).
#[inline]
fn height_to_world_y(value: f64, min_val: f64, z_scale: f32) -> f32 {
    ((value - min_val) * z_scale as f64) as f32
}

/// World Y for the water plane at `water_level`, optionally lowered to avoid z-fighting.
#[inline]
fn water_level_to_world_y(
    water_level: f64,
    min_val: f64,
    range: f64,
    z_scale: f32,
    depth_bias_frac: f32,
) -> f32 {
    let nominal = ((water_level - min_val).clamp(0.0, range) * z_scale as f64) as f32;
    let bias = (range * z_scale as f64) as f32 * depth_bias_frac.max(0.0);
    nominal - bias
}

/// Builds a centered terrain mesh (Y-up, XZ ground plane) with UVs for the colormap.
pub fn build_heightmap_mesh(
    grid: &[Vec<f64>],
    step: usize,
    xy_scale: f32,
    z_scale: f32,
) -> HeightmapMesh {
    let step = step.max(1);
    let src_h = grid.len();
    let src_w = grid[0].len();
    let mesh_h = downsample_dims(src_h, step);
    let mesh_w = downsample_dims(src_w, step);
    let (min_val, _, _) = grid_min_max(grid);

    let cx = (mesh_w - 1) as f32 * xy_scale * 0.5;
    let cz = (mesh_h - 1) as f32 * xy_scale * 0.5;
    let u_den = (mesh_w - 1).max(1) as f32;
    let v_den = (mesh_h - 1).max(1) as f32;

    let mut vertices = Vec::with_capacity(mesh_h * mesh_w);
    let mut uvs = Vec::with_capacity(mesh_h * mesh_w);
    for yi in 0..mesh_h {
        let sy = (yi * step).min(src_h - 1);
        for xi in 0..mesh_w {
            let sx = (xi * step).min(src_w - 1);
            let h = height_to_world_y(grid[sy][sx], min_val, z_scale);
            vertices.push(Vec3::new(
                xi as f32 * xy_scale - cx,
                h,
                yi as f32 * xy_scale - cz,
            ));
            uvs.push(Vec2::new(xi as f32 / u_den, yi as f32 / v_den));
        }
    }

    let mut faces = Vec::with_capacity((mesh_h - 1) * (mesh_w - 1) * 2);
    for y in 0..mesh_h - 1 {
        for x in 0..mesh_w - 1 {
            let i = (y * mesh_w + x) as VertexIndex;
            let i_right = i + 1;
            let i_down = i + mesh_w as VertexIndex;
            let i_diag = i_down + 1;
            faces.push([i, i_down, i_right]);
            faces.push([i_right, i_down, i_diag]);
        }
    }

    HeightmapMesh {
        vertices,
        faces,
        uvs,
    }
}

/// Horizontal XZ quad at a fixed world Y (same axis as terrain vertex heights).
fn build_water_plane_mesh(half_w: f32, half_d: f32, water_y: f32) -> (Vec<Vec3>, Vec<[VertexIndex; 3]>) {
    let vertices = vec![
        Vec3::new(-half_w, water_y, -half_d),
        Vec3::new(half_w, water_y, -half_d),
        Vec3::new(half_w, water_y, half_d),
        Vec3::new(-half_w, water_y, half_d),
    ];
    let faces = vec![[0, 1, 2], [0, 2, 3]];
    (vertices, faces)
}

/// Opens a window and renders `grid` as a heightmap. Blocks until the window closes.
///
/// When `show_water_plane` is true, a flat blue quad is drawn at `water_level` in heightmap units
/// (same threshold as the blue areas on the terrain texture).
pub async fn view_heightmap(
    grid: &[Vec<f64>],
    water_level: f64,
    show_water_plane: bool,
    config: HeightmapViewConfig,
) {
    let (min_val, _, range) = grid_min_max(grid);
    let mesh = build_heightmap_mesh(grid, config.step, config.xy_scale, config.z_scale);
    let colormap = height_colormap_png(grid, config.step, water_level);

    let gpu_mesh = Rc::new(RefCell::new(GpuMesh3d::new(
        mesh.vertices,
        mesh.faces,
        None,
        Some(mesh.uvs),
        false,
    )));

    let extent = {
        let w = downsample_dims(grid[0].len(), config.step);
        let h = downsample_dims(grid.len(), config.step);
        (w.max(h) as f32) * config.xy_scale
    };
    let cam_dist = extent * 1.4 + 20.0;

    let mut window = Window::new("fallen_leaves_world — heightmap").await;
    window.set_ambient(0.15);
    window.set_background_color(Color::new(0.45, 0.62, 0.82, 1.0));

    let mut camera = OrbitCamera3d::new(
        Vec3::new(cam_dist * 0.55, cam_dist * 0.45, cam_dist * 0.55),
        Vec3::ZERO,
    );
    let mut scene = SceneNode3d::empty();

    scene
        .add_light(
            Light::directional(Vec3::new(0.35, -1.0, 0.25))
                .with_intensity(2.2)
                .with_color(Color::new(1.0, 0.98, 0.92, 1.0)),
        )
        .set_position(Vec3::new(0.0, cam_dist * 1.5, 0.0));

    scene
        .add_light(Light::point(cam_dist * 2.0).with_intensity(0.35))
        .set_position(Vec3::new(-cam_dist * 0.4, cam_dist * 0.3, cam_dist * 0.5));

    scene
        .add_mesh(gpu_mesh, Vec3::ONE)
        .set_color(WHITE)
        .set_texture_from_memory(&colormap, "terrain_height")
        .set_metallic(0.0)
        .set_roughness(0.92)
        .enable_backface_culling(true);

    // Terrain first, then water slightly below nominal level (kiss3d has no per-object depth bias).
    if show_water_plane {
        let mesh_w = downsample_dims(grid[0].len(), config.step);
        let mesh_h = downsample_dims(grid.len(), config.step);
        let half_w = (mesh_w - 1).max(1) as f32 * config.xy_scale * 0.5;
        let half_d = (mesh_h - 1).max(1) as f32 * config.xy_scale * 0.5;
        let water_y = water_level_to_world_y(
            water_level,
            min_val,
            range,
            config.z_scale,
            config.water_depth_bias_frac,
        );
        let (water_verts, water_faces) = build_water_plane_mesh(half_w, half_d, water_y);
        let water_mesh = Rc::new(RefCell::new(GpuMesh3d::new(
            water_verts,
            water_faces,
            None,
            None,
            false,
        )));

        scene
            .add_mesh(water_mesh, Vec3::ONE)
            .set_color(Color::new(20.0 / 255.0, 66.0 / 255.0, 114.0 / 255.0, 1.0))
            .set_metallic(0.0)
            .set_roughness(0.35)
            .enable_backface_culling(false);
    }

    while window.render_3d(&mut scene, &mut camera).await {}
}
