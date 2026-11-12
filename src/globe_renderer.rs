use image::{DynamicImage, ImageBuffer, Rgb};
use kiss3d::prelude::*;
use kiss3d::resource::TextureManager;
use kiss3d::resource::vertex_index::VertexIndex;

use crate::helpers;
use crate::worldgen_pipeline::HexWorldPipelineRenderStruct;

const HEIGHT_RAMP_TEX: &str = "hex_height_ramp";
const HEX_FAN_MAX_WS: i32 = 80;
const CHUNK_COUNT: usize = 20;

#[derive(Default, Copy, Clone, PartialEq)]
pub enum RenderMode {
    #[default]
    PureSimplex,
    HexGlobe,
}

struct HexChunks {
    nodes: [SceneNode3d; CHUNK_COUNT],
    normals: [[f32; 3]; CHUNK_COUNT],
}

enum ActiveView {
    Simplex,
    Hex(HexChunks),
}

fn globe_radius(pipeline: &HexWorldPipelineRenderStruct) -> f32 {
    pipeline.ws as f32 * 0.4
}

fn orbit_camera(eye_z: f32) -> OrbitCamera3d {
    OrbitCamera3d::new_with_frustum(std::f32::consts::FRAC_PI_4, 0.1, (eye_z * 8.0).max(1000.0), Vec3::new(0.0, 0.0, eye_z), Vec3::ZERO)
}

fn add_globe_lights(scene: &mut SceneNode3d) {
    scene
        .add_light(
            Light::directional(Vec3::new(0.35, -1.0, 0.25))
                .with_intensity(10.0)
                .with_color(Color::new(1.0, 1.0, 1.0, 1.0)),
        )
        .set_position(Vec3::new(0.0, 3.0, 0.0));
}

fn simplex_dots_gpu_mesh(pipeline: &HexWorldPipelineRenderStruct) -> Rc<RefCell<GpuMesh3d>> {
    let n = pipeline.n as usize;
    let noise_effect_coeff = 0.25;
    let dot_size = 0.003;
    let mut coords = Vec::with_capacity(n * 6);
    let mut faces = Vec::with_capacity(n * 8);

    for idx in pipeline.start_idx_pure_noise..=pipeline.end_idx_pure_noise {
        let i = idx as usize;
        let base = i * 3;
        let x = pipeline.cell_unit_sphere_pos[base];
        let y = pipeline.cell_unit_sphere_pos[base + 1];
        let z = pipeline.cell_unit_sphere_pos[base + 2];
        let r = 1.0 + noise_effect_coeff * pipeline.global_noise_data[i];
        let cx = x * r;
        let cy = y * r;
        let cz = z * r;
        let vert_idx_base = coords.len() as VertexIndex;
        coords.push(Vec3::new(cx + dot_size, cy, cz));
        coords.push(Vec3::new(cx - dot_size, cy, cz));
        coords.push(Vec3::new(cx, cy + dot_size, cz));
        coords.push(Vec3::new(cx, cy - dot_size, cz));
        coords.push(Vec3::new(cx, cy, cz + dot_size));
        coords.push(Vec3::new(cx, cy, cz - dot_size));
        faces.push([vert_idx_base, vert_idx_base + 2, vert_idx_base + 4]);
        faces.push([vert_idx_base, vert_idx_base + 4, vert_idx_base + 3]);
        faces.push([vert_idx_base, vert_idx_base + 3, vert_idx_base + 5]);
        faces.push([vert_idx_base, vert_idx_base + 5, vert_idx_base + 2]);
        faces.push([vert_idx_base + 1, vert_idx_base + 4, vert_idx_base + 2]);
        faces.push([vert_idx_base + 1, vert_idx_base + 2, vert_idx_base + 5]);
        faces.push([vert_idx_base + 1, vert_idx_base + 5, vert_idx_base + 3]);
        faces.push([vert_idx_base + 1, vert_idx_base + 3, vert_idx_base + 4]);
    }

    Rc::new(RefCell::new(GpuMesh3d::new(coords, faces, None, None, false)))
}

fn spawn_pure_simplex_view(scene: &mut SceneNode3d, camera: &mut OrbitCamera3d, pipeline: &HexWorldPipelineRenderStruct) {
    add_globe_lights(scene);
    scene
        .add_light(Light::point(6.0).with_intensity(0.45))
        .set_position(Vec3::new(-2.0, 1.5, 2.5));
    let radius = globe_radius(pipeline);
    *camera = orbit_camera(3.0 * radius);
    scene
        .add_mesh(simplex_dots_gpu_mesh(pipeline), Vec3::splat(radius))
        .set_color(Color::new(1.0, 1.0, 1.0, 1.0))
        .set_metallic(0.0)
        .set_roughness(0.92);
}

fn spawn_chunk_nodes(scene: &mut SceneNode3d, meshes: [Rc<RefCell<GpuMesh3d>>; CHUNK_COUNT], scale: f32) -> HexChunks {
    let normals = helpers::ico_face_normals();
    let nodes = core::array::from_fn(|i| {
        scene
            .add_mesh(meshes[i].clone(), Vec3::splat(scale))
            .set_color(Color::new(1.0, 1.0, 1.0, 1.0))
            .set_metallic(0.0)
            .set_roughness(0.92)
            .enable_backface_culling(true)
    });
    HexChunks { nodes, normals }
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

fn push_gpu_mesh(coords: Vec<Vec3>, faces: Vec<[VertexIndex; 3]>, uvs: Vec<Vec2>, normals: Option<Vec<Vec3>>) -> Rc<RefCell<GpuMesh3d>> {
    Rc::new(RefCell::new(GpuMesh3d::new(coords, faces, normals, Some(uvs), false)))
}

fn hex_fan_chunk_meshes(pipeline: &HexWorldPipelineRenderStruct, cell_face: &[u8]) -> [Rc<RefCell<GpuMesh3d>>; CHUNK_COUNT] {
    let n = pipeline.n as usize;
    let verts = &pipeline.hex_verts;
    let tans = &pipeline.norm_height;
    let idx = &pipeline.tri_faces_idxs;
    let mut coords = vec![Vec::new(); CHUNK_COUNT];
    let mut uvs = vec![Vec::new(); CHUNK_COUNT];
    let mut faces = vec![Vec::new(); CHUNK_COUNT];
    let mut remap = vec![-1i32; n * 7];

    for cell in 0..n {
        let chunk = cell_face[cell] as usize;
        let cnt = pipeline.neighbor_cnt[cell] as usize;
        let remap_vert = |v: u32, coords: &mut [Vec<Vec3>], uvs: &mut [Vec<Vec2>], remap: &mut [i32]| -> VertexIndex {
            let v = v as usize;
            if remap[v] >= 0 {
                return remap[v] as VertexIndex;
            }
            let dst = coords[chunk].len() as i32;
            remap[v] = dst;
            let o = v * 3;
            coords[chunk].push(Vec3::new(verts[o], verts[o + 1], verts[o + 2]));
            uvs[chunk].push(Vec2::new(tans[v], 0.5));
            dst as VertexIndex
        };
        for t in 0..cnt {
            let base = cell * 18 + t * 3;
            let a = idx[base];
            let b = idx[base + 1];
            let c = idx[base + 2];
            if a == b && b == c {
                continue;
            }
            let ia = remap_vert(a, &mut coords, &mut uvs, &mut remap);
            let ib = remap_vert(b, &mut coords, &mut uvs, &mut remap);
            let ic = remap_vert(c, &mut coords, &mut uvs, &mut remap);
            faces[chunk].push([ia, ib, ic]);
        }
        for v in cell * 7..cell * 7 + 7 {
            remap[v] = -1;
        }
    }

    core::array::from_fn(|i| push_gpu_mesh(core::mem::take(&mut coords[i]), core::mem::take(&mut faces[i]), core::mem::take(&mut uvs[i]), None))
}

fn compact_chunk_meshes(pipeline: &HexWorldPipelineRenderStruct, cell_face: &[u8]) -> [Rc<RefCell<GpuMesh3d>>; CHUNK_COUNT] {
    let n = pipeline.n as usize;
    let mut buckets: [Vec<[u32; 3]>; CHUNK_COUNT] = core::array::from_fn(|_| Vec::new());
    for i in 0..n {
        let cnt = pipeline.neighbor_cnt[i] as usize;
        let nb = i * 6;
        for k in 0..cnt {
            let a = pipeline.neighbor_ids[nb + k] as u32;
            let b = pipeline.neighbor_ids[nb + (k + 1) % cnt] as u32;
            if (i as u32) < a && (i as u32) < b {
                let i0 = i as u32;
                let p0 = [
                    pipeline.cell_unit_sphere_pos[i * 3],
                    pipeline.cell_unit_sphere_pos[i * 3 + 1],
                    pipeline.cell_unit_sphere_pos[i * 3 + 2],
                ];
                let p1 = [
                    pipeline.cell_unit_sphere_pos[a as usize * 3],
                    pipeline.cell_unit_sphere_pos[a as usize * 3 + 1],
                    pipeline.cell_unit_sphere_pos[a as usize * 3 + 2],
                ];
                let p2 = [
                    pipeline.cell_unit_sphere_pos[b as usize * 3],
                    pipeline.cell_unit_sphere_pos[b as usize * 3 + 1],
                    pipeline.cell_unit_sphere_pos[b as usize * 3 + 2],
                ];
                let e1 = [p1[0] - p0[0], p1[1] - p0[1], p1[2] - p0[2]];
                let e2 = [p2[0] - p0[0], p2[1] - p0[1], p2[2] - p0[2]];
                let cross = [
                    e1[1] * e2[2] - e1[2] * e2[1],
                    e1[2] * e2[0] - e1[0] * e2[2],
                    e1[0] * e2[1] - e1[1] * e2[0],
                ];
                let outward = cross[0] * p0[0] + cross[1] * p0[1] + cross[2] * p0[2];
                let tri = if outward < 0.0 { [i0, b, a] } else { [i0, a, b] };
                buckets[cell_face[i] as usize].push(tri);
            }
        }
    }

    core::array::from_fn(|chunk| {
        let tris = &buckets[chunk];
        let mut remap = vec![-1i32; n];
        let mut coords = Vec::with_capacity(tris.len());
        let mut uvs = Vec::with_capacity(tris.len());
        let mut faces = Vec::with_capacity(tris.len());
        for tri in tris {
            let mut out = [0 as VertexIndex; 3];
            for k in 0..3 {
                let v = tri[k] as usize;
                if remap[v] < 0 {
                    remap[v] = coords.len() as i32;
                    let o = v * 3;
                    coords.push(Vec3::new(
                        pipeline.cell_unit_sphere_pos[o],
                        pipeline.cell_unit_sphere_pos[o + 1],
                        pipeline.cell_unit_sphere_pos[o + 2],
                    ));
                    uvs.push(Vec2::new(pipeline.norm_height[v * 7], 0.5));
                }
                out[k] = remap[v] as VertexIndex;
            }
            faces.push(out);
        }
        let normals = coords.clone();
        push_gpu_mesh(coords, faces, uvs, Some(normals))
    })
}

fn spawn_hex_globe_view(scene: &mut SceneNode3d, camera: &mut OrbitCamera3d, pipeline: &HexWorldPipelineRenderStruct) -> HexChunks {
    add_globe_lights(scene);
    let radius = globe_radius(pipeline);
    *camera = orbit_camera(3.0 * radius);

    TextureManager::get_global_manager(|tm| {
        tm.add_image(height_ramp_image(), HEIGHT_RAMP_TEX);
    });

    let mut cell_face = vec![0u8; pipeline.n as usize];
    helpers::gen_cell_ico_face_data(&mut cell_face, pipeline.n, pipeline.ws);
    let meshes = if pipeline.ws <= HEX_FAN_MAX_WS {
        hex_fan_chunk_meshes(pipeline, &cell_face)
    } else {
        compact_chunk_meshes(pipeline, &cell_face)
    };
    let mut chunks = spawn_chunk_nodes(scene, meshes, radius);
    for node in &mut chunks.nodes {
        node.set_texture_with_name(HEIGHT_RAMP_TEX);
    }
    chunks
}

fn cull_hex_chunks(chunks: &mut HexChunks, eye: Vec3) {
    let eye_len = eye.length();
    let keep = -0.35 * eye_len;
    for i in 0..CHUNK_COUNT {
        let n = chunks.normals[i];
        let facing = n[0] * eye.x + n[1] * eye.y + n[2] * eye.z;
        chunks.nodes[i].set_visible(facing > keep);
    }
}

fn spawn_curr_mode(curr_mode: RenderMode, scene: &mut SceneNode3d, camera: &mut OrbitCamera3d, pipeline: &HexWorldPipelineRenderStruct) -> ActiveView {
    match curr_mode {
        RenderMode::PureSimplex => {
            spawn_pure_simplex_view(scene, camera, pipeline);
            ActiveView::Simplex
        }
        RenderMode::HexGlobe => ActiveView::Hex(spawn_hex_globe_view(scene, camera, pipeline)),
    }
}

pub async fn render_hex_pipeline(pipeline: &HexWorldPipelineRenderStruct) {
    let mut window = Window::new("globe_render").await;
    window.set_ambient(0.25);
    let mut camera = orbit_camera(3.0);
    let mut scene = SceneNode3d::empty();

    let mut render_mode = RenderMode::HexGlobe;
    let mut view = spawn_curr_mode(render_mode, &mut scene, &mut camera, pipeline);

    let switch_render_mode = |curr_mode: RenderMode| {
        if curr_mode == RenderMode::PureSimplex {
            RenderMode::HexGlobe
        } else {
            RenderMode::PureSimplex
        }
    };

    while window.render_3d(&mut scene, &mut camera).await {
        if let ActiveView::Hex(chunks) = &mut view {
            cull_hex_chunks(chunks, camera.eye());
        }
        for event in window.events().iter() {
            match event.value {
                WindowEvent::Key(Key::G, Action::Press, _) => {
                    render_mode = switch_render_mode(render_mode);
                    scene = SceneNode3d::empty();
                    view = spawn_curr_mode(render_mode, &mut scene, &mut camera, pipeline);
                }
                _ => {}
            }
        }
    }
}
