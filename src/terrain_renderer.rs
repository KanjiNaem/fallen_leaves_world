//! Interactive 3D terrain preview (kiss3d).
//!
//! Per-cell colors are applied via UV-mapped colormap textures (kiss3d 0.41 has no
//! vertex-color channel on `GpuMesh3d`).

use crate::assign_biome::Biomes;
use crate::img_gen::LandElevationPalette;
use image::{ImageBuffer, Rgb};
use kiss3d::prelude::*;
use kiss3d::resource::TextureManager;
use kiss3d::resource::vertex_index::VertexIndex;
use std::cell::RefCell;
use std::io::Cursor;
use std::rc::Rc;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum TerrainRenderMode {
    #[default]
    Smooth,
    TrueGrid,
}

impl TerrainRenderMode {
    pub fn toggle(self) -> Self {
        match self {
            Self::Smooth => Self::TrueGrid,
            Self::TrueGrid => Self::Smooth,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Smooth => "smooth",
            Self::TrueGrid => "true grid",
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum TerrainColorMode {
    #[default]
    Height,
    Biome,
    WarpedInfluence,
    ChaoticInfluence,
}

impl TerrainColorMode {
    pub fn label(self) -> &'static str {
        match self {
            Self::Height => "height",
            Self::Biome => "biome",
            Self::WarpedInfluence => "warped",
            Self::ChaoticInfluence => "chaotic",
        }
    }

    fn texture_name(self) -> &'static str {
        match self {
            Self::Height => "terrain_height",
            Self::Biome => "terrain_biome",
            Self::WarpedInfluence => "terrain_warped",
            Self::ChaoticInfluence => "terrain_chaotic",
        }
    }
}

pub struct TerrainRendererConfig {
    pub step: usize,
    pub xy_scale: f32,
    pub z_scale: f32,
    pub water_depth_bias_frac: f32,
    pub render_mode: TerrainRenderMode,
    pub color_mode: TerrainColorMode,
}

impl Default for TerrainRendererConfig {
    fn default() -> Self {
        Self {
            step: 5,
            xy_scale: 1.0,
            z_scale: 0.1,
            water_depth_bias_frac: 0.012,
            render_mode: TerrainRenderMode::Smooth,
            color_mode: TerrainColorMode::Height,
        }
    }
}

pub struct TerrainMesh {
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

fn encode_png(img: &ImageBuffer<Rgb<u8>, Vec<u8>>) -> Vec<u8> {
    let mut png = Vec::new();
    img.write_to(&mut Cursor::new(&mut png), image::ImageFormat::Png)
        .expect("encode colormap png");
    png
}

fn lerp_u8(a: u8, b: u8, t: f32) -> u8 {
    let t = t.clamp(0.0, 1.0);
    (a as f32 + (b as f32 - a as f32) * t).round() as u8
}

fn rgb_lerp(a: [u8; 3], b: [u8; 3], t: f32) -> Rgb<u8> {
    Rgb([
        lerp_u8(a[0], b[0], t),
        lerp_u8(a[1], b[1], t),
        lerp_u8(a[2], b[2], t),
    ])
}

/// Base biomes shown in the on-screen legend (warped/chaotic share these colors).
const BIOME_LEGEND: &[(&str, Biomes)] = &[
    ("Grasslands", Biomes::GRASSLANDS),
    ("Desert", Biomes::DESERT),
    ("Ashlands", Biomes::ASHLANDS),
    ("Savannah", Biomes::SAVANNAH),
    ("Steep Mountain", Biomes::STEEP_MOUNTAIN),
    ("Mountain Peak", Biomes::MOUNTAIN_PEAK),
    ("Ocean", Biomes::OCEAN),
    ("Deep Ocean", Biomes::DEEP_OCEAN),
    ("Shallow Ocean", Biomes::SHALLOW_OCEAN),
    ("Pond", Biomes::POND),
    ("Wet Sinkhole", Biomes::WET_SINKHOLE),
    ("Lagoon", Biomes::LAGOON),
    ("Great Lagoon", Biomes::GREAT_LAGOON),
    ("Lake", Biomes::LAKE),
    ("Deep Lake", Biomes::DEEP_LAKE),
    ("Frozen Lake", Biomes::FROZEN_LAKE),
    ("Frozen Deep Lake", Biomes::FROZEN_DEEP_LAKE),
    ("Great Lake", Biomes::GREAT_LAKE),
    ("Deep Great Lake", Biomes::DEEP_GREAT_LAKE),
    ("Frozen Great Lake", Biomes::FROZEN_GREAT_LAKE),
    ("Frozen Deep Great Lake", Biomes::FROZEN_DEEP_GREAT_LAKE),
    ("Badlands", Biomes::BADLANDS),
    ("Forest", Biomes::FORREST),
    ("Sparse Forest", Biomes::SPARCE_FORREST),
    ("Deep Forest", Biomes::DEEP_FORREST),
    ("Rainforest", Biomes::RAINFORREST),
    ("Swampy Rainforest", Biomes::SWAMPY_RAINFORREST),
    ("Swampy Forest", Biomes::SWAMPY_FORREST),
    ("Bog", Biomes::BOG),
    ("Marsh", Biomes::MARSH),
    ("Swamp", Biomes::SWAMP),
    ("Ice Taiga", Biomes::ICE_TAIGA),
    ("Snowy Tundra", Biomes::SNOWY_TUNDRA),
    ("Boreal Forest", Biomes::BOREAL_FORREST),
    ("Rocky Fields", Biomes::ROCKY_FIELDS),
    ("Sandy Coast", Biomes::SANDY_COAST),
    ("Rocky Coast", Biomes::ROCKY_COAST),
    ("Macrotidal Shingle", Biomes::MACROTIDAL_SHINGLE_BEACH),
    ("Scorched Beach", Biomes::SCORCHED_BEACH),
];

fn rgb_u8_to_color([r, g, b]: [u8; 3]) -> Color {
    Color::new(r as f32 / 255.0, g as f32 / 255.0, b as f32 / 255.0, 1.0)
}

fn draw_biome_legend(window: &mut Window, font: &std::sync::Arc<Font>) {
    const COLS: usize = 2;
    const ROW_H: f32 = 22.0;
    const COL_W: f32 = 240.0;
    const TEXT_SCALE: f32 = 22.0;
    const PAD: f32 = 12.0;

    let rows = BIOME_LEGEND.len().div_ceil(COLS);
    let legend_w = COLS as f32 * COL_W + PAD;
    let origin_x = window.width() as f32 - legend_w - 16.0;
    let origin_y = 48.0;

    window.draw_text(
        "Biome legend",
        Vec2::new(origin_x + PAD, origin_y + 4.0),
        TEXT_SCALE + 2.0,
        font,
        WHITE,
    );

    for (i, &(name, biome)) in BIOME_LEGEND.iter().enumerate() {
        let col = i / rows;
        let row = i % rows;
        let x = origin_x + PAD + col as f32 * COL_W;
        let y = origin_y + PAD + 22.0 + row as f32 * ROW_H;
        let color = rgb_u8_to_color(biome_color(biome));
        // Colored swatch + name via text (2D points use a different camera space).
        window.draw_text("###", Vec2::new(x, y), TEXT_SCALE, font, color);
        window.draw_text(
            name,
            Vec2::new(x + 42.0, y),
            TEXT_SCALE,
            font,
            color,
        );
    }
}

/// Thematic color for a biome. Warped/chaotic variants share the base biome color.
pub fn biome_color(biome: Biomes) -> [u8; 3] {
    match biome {
        Biomes::GRASSLANDS | Biomes::WARPED_GRASSLANDS | Biomes::CHAOTIC_GRASSLANDS => {
            [70, 200, 55]
        }
        Biomes::DESERT | Biomes::WARPED_DESERT | Biomes::CHAOTIC_DESERT => [235, 200, 70],
        Biomes::ASHLANDS | Biomes::WARPED_ASHLANDS | Biomes::CHAOTIC_ASHLANDS => [120, 55, 40],
        Biomes::SAVANNAH | Biomes::WARPED_SAVANNAH | Biomes::CHAOTIC_SAVANNAH => [210, 185, 45],
        Biomes::STEEP_MOUNTAIN | Biomes::WARPED_STEEP_MOUNTAIN | Biomes::CHAOTIC_STEEP_MOUNTAIN => {
            [130, 95, 70]
        }
        Biomes::MOUNTAIN_PEAK | Biomes::WARPED_MOUNTAIN_PEAK | Biomes::CHAOTIC_MOUNTAIN_PEAK => {
            [245, 250, 255]
        }
        Biomes::OCEAN | Biomes::WARPED_OCEAN | Biomes::CHAOTIC_OCEAN => [25, 100, 200],
        Biomes::DEEP_OCEAN | Biomes::WARPED_DEEP_OCEAN | Biomes::CHAOTIC_DEEP_OCEAN => {
            [10, 30, 110]
        }
        Biomes::SHALLOW_OCEAN | Biomes::WARPED_SHALLOW_OCEAN | Biomes::CHAOTIC_SHALLOW_OCEAN => {
            [60, 185, 225]
        }
        Biomes::POND | Biomes::WARPED_POND | Biomes::CHAOTIC_POND => [45, 180, 140],
        Biomes::WET_SINKHOLE | Biomes::WARPED_WET_SINKHOLE | Biomes::CHAOTIC_WET_SINKHOLE => {
            [30, 110, 95]
        }
        Biomes::LAGOON | Biomes::WARPED_LAGOON | Biomes::CHAOTIC_LAGOON => [35, 200, 190],
        Biomes::GREAT_LAGOON | Biomes::WARPED_GREAT_LAGOON | Biomes::CHAOTIC_GREAT_LAGOON => {
            [25, 160, 180]
        }
        Biomes::LAKE | Biomes::WARPED_LAKE | Biomes::CHAOTIC_LAKE => [45, 130, 220],
        Biomes::DEEP_LAKE | Biomes::WARPED_DEEP_LAKE | Biomes::CHAOTIC_DEEP_LAKE => [20, 70, 160],
        Biomes::FROZEN_LAKE | Biomes::WARPED_FROZEN_LAKE | Biomes::CHAOTIC_FROZEN_LAKE => {
            [200, 235, 250]
        }
        Biomes::FROZEN_DEEP_LAKE
        | Biomes::WARPED_FROZEN_DEEP_LAKE
        | Biomes::CHAOTIC_FROZEN_DEEP_LAKE => [140, 200, 235],
        Biomes::GREAT_LAKE | Biomes::WARPED_GREAT_LAKE | Biomes::CHAOTIC_GREAT_LAKE => {
            [30, 115, 205]
        }
        Biomes::DEEP_GREAT_LAKE
        | Biomes::WARPED_DEEP_GREAT_LAKE
        | Biomes::CHAOTIC_DEEP_GREAT_LAKE => [15, 55, 140],
        Biomes::FROZEN_GREAT_LAKE
        | Biomes::WARPED_FROZEN_GREAT_LAKE
        | Biomes::CHAOTIC_FROZEN_GREAT_LAKE => [185, 230, 245],
        Biomes::FROZEN_DEEP_GREAT_LAKE
        | Biomes::WARPED_FROZEN_DEEP_GREAT_LAKE
        | Biomes::CHAOTIC_FROZEN_DEEP_GREAT_LAKE => [120, 185, 225],
        Biomes::BADLANDS | Biomes::WARPED_BADLANDS | Biomes::CHAOTIC_BADLANDS => [220, 95, 45],
        Biomes::FORREST | Biomes::WARPED_FORREST | Biomes::CHAOTIC_FORREST => [35, 140, 50],
        Biomes::SPARCE_FORREST | Biomes::WARPED_SPARCE_FORREST | Biomes::CHAOTIC_SPARCE_FORREST => {
            [120, 185, 75]
        }
        Biomes::DEEP_FORREST | Biomes::WARPED_DEEP_FORREST | Biomes::CHAOTIC_DEEP_FORREST => {
            [15, 80, 30]
        }
        Biomes::RAINFORREST | Biomes::WARPED_RAINFORREST | Biomes::CHAOTIC_RAINFORREST => {
            [10, 160, 70]
        }
        Biomes::SWAMPY_RAINFORREST
        | Biomes::WARPED_SWAMPY_RAINFORREST
        | Biomes::CHAOTIC_SWAMPY_RAINFORREST => [55, 135, 60],
        Biomes::SWAMPY_FORREST | Biomes::WARPED_SWAMPY_FORREST | Biomes::CHAOTIC_SWAMPY_FORREST => {
            [75, 125, 45]
        }
        Biomes::BOG | Biomes::WARPED_BOG | Biomes::CHAOTIC_BOG => [110, 105, 35],
        Biomes::MARSH | Biomes::WARPED_MARSH | Biomes::CHAOTIC_MARSH => [140, 175, 70],
        Biomes::SWAMP | Biomes::WARPED_SWAMP | Biomes::CHAOTIC_SWAMP => [90, 115, 40],
        Biomes::ICE_TAIGA | Biomes::WARPED_ICE_TAIGA | Biomes::CHAOTIC_ICE_TAIGA => [115, 185, 180],
        Biomes::SNOWY_TUNDRA | Biomes::WARPED_SNOWY_TUNDRA | Biomes::CHAOTIC_SNOWY_TUNDRA => {
            [225, 240, 250]
        }
        Biomes::BOREAL_FORREST | Biomes::WARPED_BOREAL_FORREST | Biomes::CHAOTIC_BOREAL_FORREST => {
            [30, 100, 80]
        }
        Biomes::ROCKY_FIELDS | Biomes::WARPED_ROCKY_FIELDS | Biomes::CHAOTIC_ROCKY_FIELDS => {
            [185, 135, 80]
        }
        Biomes::SANDY_COAST | Biomes::WARPED_SANDY_COAST | Biomes::CHAOTIC_SANDY_COAST => {
            [245, 225, 140]
        }
        Biomes::ROCKY_COAST | Biomes::WARPED_ROCKY_COAST | Biomes::CHAOTIC_ROCKY_COAST => {
            [160, 110, 70]
        }
        Biomes::MACROTIDAL_SHINGLE_BEACH
        | Biomes::WARPED_MACROTIDAL_SHINGLE_BEACH
        | Biomes::CHAOTIC_MACROTIDAL_SHINGLE_BEACH => [180, 150, 100],
        Biomes::SCORCHED_BEACH | Biomes::WARPED_SCORCHED_BEACH | Biomes::CHAOTIC_SCORCHED_BEACH => {
            [235, 115, 45]
        }
    }
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
    encode_png(&img)
}

pub fn biome_colormap_png(biome_map: &[Vec<Biomes>], step: usize) -> Vec<u8> {
    let step = step.max(1);
    let mesh_h = downsample_dims(biome_map.len(), step);
    let mesh_w = downsample_dims(biome_map[0].len(), step);
    let mut img: ImageBuffer<Rgb<u8>, Vec<u8>> = ImageBuffer::new(mesh_w as u32, mesh_h as u32);
    for yi in 0..mesh_h {
        let sy = (yi * step).min(biome_map.len() - 1);
        for xi in 0..mesh_w {
            let sx = (xi * step).min(biome_map[0].len() - 1);
            img.put_pixel(xi as u32, yi as u32, Rgb(biome_color(biome_map[sy][sx])));
        }
    }
    encode_png(&img)
}

fn influence_colormap_png(map: &[Vec<f64>], step: usize, high: [u8; 3]) -> Vec<u8> {
    let step = step.max(1);
    let mesh_h = downsample_dims(map.len(), step);
    let mesh_w = downsample_dims(map[0].len(), step);
    let low = [255u8, 255, 255];
    let mut img: ImageBuffer<Rgb<u8>, Vec<u8>> = ImageBuffer::new(mesh_w as u32, mesh_h as u32);
    for yi in 0..mesh_h {
        let sy = (yi * step).min(map.len() - 1);
        for xi in 0..mesh_w {
            let sx = (xi * step).min(map[0].len() - 1);
            let t = (map[sy][sx] / 100.0).clamp(0.0, 1.0) as f32;
            img.put_pixel(xi as u32, yi as u32, rgb_lerp(low, high, t));
        }
    }
    encode_png(&img)
}

#[inline]
fn height_to_world_y(value: f64, min_val: f64, z_scale: f32) -> f32 {
    ((value - min_val) * z_scale as f64) as f32
}

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

/// Smooth surface with UVs for a mesh_w × mesh_h colormap.
pub fn build_smooth_mesh(
    grid: &[Vec<f64>],
    step: usize,
    xy_scale: f32,
    z_scale: f32,
) -> TerrainMesh {
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

    TerrainMesh {
        vertices,
        faces,
        uvs,
    }
}

fn push_textured_triangle(
    vertices: &mut Vec<Vec3>,
    uvs: &mut Vec<Vec2>,
    faces: &mut Vec<[VertexIndex; 3]>,
    corners: [Vec3; 3],
    uv: Vec2,
) {
    let base = vertices.len() as VertexIndex;
    for corner in corners {
        vertices.push(corner);
        uvs.push(uv);
    }
    faces.push([base, base + 1, base + 2]);
}

fn push_textured_quad(
    vertices: &mut Vec<Vec3>,
    uvs: &mut Vec<Vec2>,
    faces: &mut Vec<[VertexIndex; 3]>,
    corners: [Vec3; 4],
    uv: Vec2,
    face_up: bool,
) {
    if face_up {
        push_textured_triangle(
            vertices,
            uvs,
            faces,
            [corners[0], corners[3], corners[2]],
            uv,
        );
        push_textured_triangle(
            vertices,
            uvs,
            faces,
            [corners[0], corners[2], corners[1]],
            uv,
        );
    } else {
        push_textured_triangle(
            vertices,
            uvs,
            faces,
            [corners[0], corners[1], corners[2]],
            uv,
        );
        push_textured_triangle(
            vertices,
            uvs,
            faces,
            [corners[0], corners[2], corners[3]],
            uv,
        );
    }
}

/// Column / true-grid mesh; each cell samples its colormap texel center.
pub fn build_column_grid_mesh(
    grid: &[Vec<f64>],
    step: usize,
    xy_scale: f32,
    z_scale: f32,
) -> TerrainMesh {
    let step = step.max(1);
    let src_h = grid.len();
    let src_w = grid[0].len();
    let mesh_h = downsample_dims(src_h, step);
    let mesh_w = downsample_dims(src_w, step);
    let (min_val, _, _) = grid_min_max(grid);

    let cx = mesh_w as f32 * xy_scale * 0.5;
    let cz = mesh_h as f32 * xy_scale * 0.5;

    let mut heights = vec![0.0f32; mesh_h * mesh_w];
    for yi in 0..mesh_h {
        let sy = (yi * step).min(src_h - 1);
        for xi in 0..mesh_w {
            let sx = (xi * step).min(src_w - 1);
            heights[yi * mesh_w + xi] = height_to_world_y(grid[sy][sx], min_val, z_scale);
        }
    }

    let mut vertices = Vec::new();
    let mut uvs = Vec::new();
    let mut faces = Vec::new();

    for yi in 0..mesh_h {
        for xi in 0..mesh_w {
            let uv = Vec2::new(
                (xi as f32 + 0.5) / mesh_w.max(1) as f32,
                (yi as f32 + 0.5) / mesh_h.max(1) as f32,
            );
            let h = heights[yi * mesh_w + xi];
            let x0 = xi as f32 * xy_scale - cx;
            let x1 = (xi + 1) as f32 * xy_scale - cx;
            let z0 = yi as f32 * xy_scale - cz;
            let z1 = (yi + 1) as f32 * xy_scale - cz;

            push_textured_quad(
                &mut vertices,
                &mut uvs,
                &mut faces,
                [
                    Vec3::new(x0, h, z0),
                    Vec3::new(x1, h, z0),
                    Vec3::new(x1, h, z1),
                    Vec3::new(x0, h, z1),
                ],
                uv,
                true,
            );

            let h_nx = if xi == 0 {
                0.0
            } else {
                heights[yi * mesh_w + xi - 1]
            };
            if xi == 0 || h_nx < h {
                let y0 = if xi == 0 { 0.0 } else { h_nx };
                push_textured_quad(
                    &mut vertices,
                    &mut uvs,
                    &mut faces,
                    [
                        Vec3::new(x0, y0, z0),
                        Vec3::new(x0, y0, z1),
                        Vec3::new(x0, h, z1),
                        Vec3::new(x0, h, z0),
                    ],
                    uv,
                    false,
                );
            }

            let h_px = if xi + 1 >= mesh_w {
                0.0
            } else {
                heights[yi * mesh_w + xi + 1]
            };
            if xi + 1 >= mesh_w || h_px < h {
                let y0 = if xi + 1 >= mesh_w { 0.0 } else { h_px };
                push_textured_quad(
                    &mut vertices,
                    &mut uvs,
                    &mut faces,
                    [
                        Vec3::new(x1, y0, z0),
                        Vec3::new(x1, h, z0),
                        Vec3::new(x1, h, z1),
                        Vec3::new(x1, y0, z1),
                    ],
                    uv,
                    false,
                );
            }

            let h_nz = if yi == 0 {
                0.0
            } else {
                heights[(yi - 1) * mesh_w + xi]
            };
            if yi == 0 || h_nz < h {
                let y0 = if yi == 0 { 0.0 } else { h_nz };
                push_textured_quad(
                    &mut vertices,
                    &mut uvs,
                    &mut faces,
                    [
                        Vec3::new(x0, y0, z0),
                        Vec3::new(x0, h, z0),
                        Vec3::new(x1, h, z0),
                        Vec3::new(x1, y0, z0),
                    ],
                    uv,
                    false,
                );
            }

            let h_pz = if yi + 1 >= mesh_h {
                0.0
            } else {
                heights[(yi + 1) * mesh_w + xi]
            };
            if yi + 1 >= mesh_h || h_pz < h {
                let y0 = if yi + 1 >= mesh_h { 0.0 } else { h_pz };
                push_textured_quad(
                    &mut vertices,
                    &mut uvs,
                    &mut faces,
                    [
                        Vec3::new(x0, y0, z1),
                        Vec3::new(x1, y0, z1),
                        Vec3::new(x1, h, z1),
                        Vec3::new(x0, h, z1),
                    ],
                    uv,
                    false,
                );
            }
        }
    }

    TerrainMesh {
        vertices,
        faces,
        uvs,
    }
}

fn build_water_plane_mesh(
    half_w: f32,
    half_d: f32,
    water_y: f32,
) -> (Vec<Vec3>, Vec<[VertexIndex; 3]>) {
    let vertices = vec![
        Vec3::new(-half_w, water_y, -half_d),
        Vec3::new(half_w, water_y, -half_d),
        Vec3::new(half_w, water_y, half_d),
        Vec3::new(-half_w, water_y, half_d),
    ];
    let faces = vec![[0, 1, 2], [0, 2, 3]];
    (vertices, faces)
}

fn gpu_mesh_from_terrain(mesh: TerrainMesh) -> Rc<RefCell<GpuMesh3d>> {
    Rc::new(RefCell::new(GpuMesh3d::new(
        mesh.vertices,
        mesh.faces,
        None,
        Some(mesh.uvs),
        false,
    )))
}

fn style_terrain_node(node: &mut SceneNode3d, color_mode: TerrainColorMode) {
    node.set_color(WHITE)
        .set_texture_with_name(color_mode.texture_name())
        .set_metallic(0.0)
        .set_roughness(0.92)
        .enable_backface_culling(true);
}

fn apply_color_mode(node: &mut SceneNode3d, color_mode: TerrainColorMode) {
    node.set_texture_with_name(color_mode.texture_name());
}

fn water_plane_visible(render_mode: TerrainRenderMode, color_mode: TerrainColorMode) -> bool {
    render_mode == TerrainRenderMode::Smooth && color_mode == TerrainColorMode::Height
}

/// Opens a window and renders terrain. Blocks until the window closes.
pub async fn view_terrain(
    grid: &[Vec<f64>],
    biome_map: &[Vec<Biomes>],
    magic_map: &[Vec<f64>],
    chaos_map: &[Vec<f64>],
    water_level: f64,
    show_water_plane: bool,
    config: TerrainRendererConfig,
) {
    let (min_val, _, range) = grid_min_max(grid);

    let height_png = height_colormap_png(grid, config.step, water_level);
    let biome_png = biome_colormap_png(biome_map, config.step);
    let warped_png = influence_colormap_png(magic_map, config.step, [0, 20, 90]);
    let chaotic_png = influence_colormap_png(chaos_map, config.step, [110, 0, 0]);

    let smooth_mesh = build_smooth_mesh(grid, config.step, config.xy_scale, config.z_scale);
    let grid_mesh = build_column_grid_mesh(grid, config.step, config.xy_scale, config.z_scale);

    let mesh_w = downsample_dims(grid[0].len(), config.step);
    let mesh_h = downsample_dims(grid.len(), config.step);
    let extent = (mesh_w.max(mesh_h) as f32) * config.xy_scale;
    let cam_dist = extent * 1.4 + 20.0;

    // Window must exist before TextureManager is available.
    let mut window = Window::new("fallen_leaves_world — terrain").await;
    window.set_ambient(0.15);
    window.set_background_color(Color::new(0.45, 0.62, 0.82, 1.0));

    TextureManager::get_global_manager(|tm| {
        tm.add_image_from_memory(&height_png, TerrainColorMode::Height.texture_name());
        tm.add_image_from_memory(&biome_png, TerrainColorMode::Biome.texture_name());
        tm.add_image_from_memory(
            &warped_png,
            TerrainColorMode::WarpedInfluence.texture_name(),
        );
        tm.add_image_from_memory(
            &chaotic_png,
            TerrainColorMode::ChaoticInfluence.texture_name(),
        );
    });

    let mut camera = OrbitCamera3d::new(
        Vec3::new(cam_dist * 0.55, cam_dist * 0.45, cam_dist * 0.55),
        Vec3::ZERO,
    );
    let mut scene = SceneNode3d::empty();
    let font = Font::default();

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

    let mut render_mode = config.render_mode;
    let mut color_mode = config.color_mode;
    let mut show_legend = true;

    let mut smooth_node = scene.add_mesh(gpu_mesh_from_terrain(smooth_mesh), Vec3::ONE);
    style_terrain_node(&mut smooth_node, color_mode);
    smooth_node.set_visible(render_mode == TerrainRenderMode::Smooth);

    let mut grid_node = scene.add_mesh(gpu_mesh_from_terrain(grid_mesh), Vec3::ONE);
    style_terrain_node(&mut grid_node, color_mode);
    grid_node.set_visible(render_mode == TerrainRenderMode::TrueGrid);

    let mut water_node = if show_water_plane {
        let half_w = mesh_w.max(1) as f32 * config.xy_scale * 0.5;
        let half_d = mesh_h.max(1) as f32 * config.xy_scale * 0.5;
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

        let mut node = scene
            .add_mesh(water_mesh, Vec3::ONE)
            .set_color(Color::new(20.0 / 255.0, 66.0 / 255.0, 114.0 / 255.0, 1.0))
            .set_metallic(0.0)
            .set_roughness(0.35)
            .enable_backface_culling(false);
        node.set_visible(water_plane_visible(render_mode, color_mode));
        Some(node)
    } else {
        None
    };

    while window.render_3d(&mut scene, &mut camera).await {
        let mut color_changed = false;
        for event in window.events().iter() {
            match event.value {
                WindowEvent::Key(Key::G, Action::Press, _) => {
                    render_mode = render_mode.toggle();
                }
                WindowEvent::Key(Key::C, Action::Press, _) => {
                    color_mode = TerrainColorMode::Height;
                    color_changed = true;
                }
                WindowEvent::Key(Key::B, Action::Press, _) => {
                    color_mode = TerrainColorMode::Biome;
                    color_changed = true;
                }
                WindowEvent::Key(Key::W, Action::Press, _) => {
                    color_mode = TerrainColorMode::WarpedInfluence;
                    color_changed = true;
                }
                WindowEvent::Key(Key::K, Action::Press, _) => {
                    color_mode = TerrainColorMode::ChaoticInfluence;
                    color_changed = true;
                }
                WindowEvent::Key(Key::L, Action::Press, _) => {
                    show_legend = !show_legend;
                }
                _ => {}
            }
        }

        if color_changed {
            apply_color_mode(&mut smooth_node, color_mode);
            apply_color_mode(&mut grid_node, color_mode);
        }

        smooth_node.set_visible(render_mode == TerrainRenderMode::Smooth);
        grid_node.set_visible(render_mode == TerrainRenderMode::TrueGrid);
        if let Some(ref mut water) = water_node {
            water.set_visible(water_plane_visible(render_mode, color_mode));
        }

        let hud = format!(
            "render: {}  |  color: {}  |  G: geometry  |  C: height  |  B: biome  |  W: warped  |  K: chaotic  |  L: legend",
            render_mode.label(),
            color_mode.label()
        );
        window.draw_text(&hud, Vec2::new(12.0, 12.0), 36.0, &font, WHITE);

        if show_legend {
            draw_biome_legend(&mut window, &font);
        }
    }
}
