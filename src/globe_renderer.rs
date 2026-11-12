use image::{DynamicImage, ImageBuffer, Rgb};
use kiss3d::glamx::Mat4;
use kiss3d::prelude::*;
use kiss3d::resource::TextureManager;
use kiss3d::resource::vertex_index::VertexIndex;

use crate::helpers;
use crate::worldgen_pipeline::HexWorldPipelineRenderStruct;

const HEIGHT_RAMP_TEX: &str = "hex_height_ramp";
const HEX_FAN_MAX_WS: i32 = 200;
const DETAIL_HEX_RADIUS: i32 = 150;
const DETAIL_SHELL: f32 = 1.0002;
const CHUNK_COUNT: usize = 20;
const SPIN_STEP: f32 = 0.005;
const DIST_STEP: f32 = 1.01;
const HOVER_EDGE_SHELL: f32 = 1.002;
const HOVER_TEXT_SCALE: f32 = 48.0;

#[derive(Default, Copy, Clone, PartialEq)]
pub enum RenderMode {
    #[default]
    PureSimplex,
    HexGlobe,
}

struct HexChunks {
    nodes: [SceneNode3d; CHUNK_COUNT],
    normals: [[f32; 3]; CHUNK_COUNT],
    detail: Option<DetailLod>,
}

struct DetailLod {
    node: Option<SceneNode3d>,
    center: i32,
    seen: Vec<u32>,
    seen_gen: u32,
    order: Vec<i32>,
    dist: Vec<u16>,
}

enum ActiveView {
    Simplex,
    Hex(HexChunks),
}

struct LiveView {
    rig: SceneNode3d,
    kind: ActiveView,
    hover_cell: Option<i32>,
}

struct GlobeCamera<'a> {
    orbit: OrbitCamera3d,
    dragging: bool,
    seen_cursor: bool,
    cursor: Vec2,
    view: Rc<RefCell<LiveView>>,
    pipeline: &'a HexWorldPipelineRenderStruct,
}

impl<'a> GlobeCamera<'a> {
    fn new(
        radius: f32,
        view: Rc<RefCell<LiveView>>,
        pipeline: &'a HexWorldPipelineRenderStruct,
    ) -> Self {
        let eye_z = 3.0 * radius;
        let znear = (radius * 0.02).max(0.01);
        let zfar = (radius * 80.0).max(1000.0);
        let mut orbit = OrbitCamera3d::new_with_frustum(
            std::f32::consts::FRAC_PI_4,
            znear,
            zfar,
            Vec3::new(0.0, 0.0, eye_z),
            Vec3::ZERO,
        );
        orbit.rebind_rotate_button(None);
        orbit.rebind_drag_button(None);
        orbit.rebind_reset_key(None);
        let mut camera = Self {
            orbit,
            dragging: false,
            seen_cursor: false,
            cursor: Vec2::ZERO,
            view,
            pipeline,
        };
        camera.frame_radius(radius);
        camera
    }

    fn frame_radius(&mut self, radius: f32) {
        let eye_z = 3.0 * radius;
        self.orbit.set_min_dist(radius * 1.25);
        self.orbit.set_max_dist(radius * 40.0);
        self.orbit.set_dist(eye_z);
        self.orbit.set_at(Vec3::ZERO);
    }

    fn spin(&mut self, delta: Vec2) {
        let spin =
            Quat::from_rotation_y(delta.x * SPIN_STEP) * Quat::from_rotation_x(delta.y * SPIN_STEP);
        let mut view = self.view.borrow_mut();
        view.rig.rotate(spin);
        view.rig.set_position(Vec3::ZERO);
    }
}

impl<'a> Camera3d for GlobeCamera<'a> {
    fn handle_event(&mut self, canvas: &Canvas, event: &WindowEvent) {
        match *event {
            WindowEvent::Scroll(_, off, _) => {
                self.orbit
                    .set_dist(self.orbit.dist() * DIST_STEP.powf(off as f32));
                self.orbit.set_at(Vec3::ZERO);
            }
            WindowEvent::FramebufferSize(_, _) => {
                self.orbit.handle_event(canvas, event);
                self.orbit.set_at(Vec3::ZERO);
            }
            WindowEvent::MouseButton(MouseButton::Button1, Action::Press, _) => {
                self.dragging = true;
            }
            WindowEvent::MouseButton(MouseButton::Button1, Action::Release, _) => {
                self.dragging = false;
            }
            WindowEvent::CursorPos(x, y, _) => {
                let curr = Vec2::new(x as f32, y as f32);
                if self.dragging && self.seen_cursor {
                    self.spin(curr - self.cursor);
                }
                self.cursor = curr;
                self.seen_cursor = true;
            }
            _ => {}
        }
    }

    fn eye(&self) -> Vec3 {
        self.orbit.eye()
    }

    fn view_transform(&self) -> Pose3 {
        self.orbit.view_transform()
    }

    fn transformation(&self) -> Mat4 {
        self.orbit.transformation()
    }

    fn inverse_transformation(&self) -> Mat4 {
        self.orbit.inverse_transformation()
    }

    fn clip_planes(&self) -> (f32, f32) {
        self.orbit.clip_planes()
    }

    fn update(&mut self, canvas: &Canvas) {
        self.orbit.update(canvas);
        let eye = self.orbit.eye();
        let pick = if self.seen_cursor {
            let (w, h) = canvas.size();
            Some(self.unproject(self.cursor, Vec2::new(w as f32, h as f32)))
        } else {
            None
        };
        let mut view = self.view.borrow_mut();
        sync_view(&mut view, eye, self.pipeline);
        sync_hover(&mut view, pick, self.pipeline);
    }

    fn view_transform_pair(&self, pass: usize) -> (Pose3, Mat4) {
        self.orbit.view_transform_pair(pass)
    }
}

fn globe_radius(pipeline: &HexWorldPipelineRenderStruct) -> f32 {
    pipeline.ws as f32 * 0.4
}

fn add_globe_lights(scene: &mut SceneNode3d) {
    let key = Vec3::new(0.35, -1.0, 0.25);
    let color = Color::new(1.0, 1.0, 1.0, 1.0);
    scene.add_light(
        Light::directional(key)
            .with_intensity(0.6)
            .with_color(color),
    );
    scene.add_light(
        Light::directional(-key)
            .with_intensity(0.6)
            .with_color(color),
    );
}

fn style_hex_node(node: &mut SceneNode3d) {
    node.set_color(Color::new(1.0, 1.0, 1.0, 1.0))
        .set_metallic(0.0)
        .set_roughness(0.92)
        .enable_backface_culling(true)
        .set_texture_with_name(HEIGHT_RAMP_TEX);
}

fn simplex_dots_gpu_mesh(pipeline: &HexWorldPipelineRenderStruct) -> Rc<RefCell<GpuMesh3d>> {
    let n = pipeline.n as usize;
    let noise_effect_coeff = 0.25;
    let dot_size = 0.05 / pipeline.ws as f32;
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

    Rc::new(RefCell::new(GpuMesh3d::new(
        coords, faces, None, None, false,
    )))
}

fn spawn_pure_simplex_view(rig: &mut SceneNode3d, pipeline: &HexWorldPipelineRenderStruct) {
    let radius = globe_radius(pipeline);
    rig.add_mesh(simplex_dots_gpu_mesh(pipeline), Vec3::splat(radius))
        .set_color(Color::new(1.0, 1.0, 1.0, 1.0))
        .set_metallic(0.0)
        .set_roughness(0.92);
}

fn spawn_chunk_nodes(
    rig: &mut SceneNode3d,
    meshes: [Rc<RefCell<GpuMesh3d>>; CHUNK_COUNT],
    scale: f32,
) -> HexChunks {
    let normals = helpers::ico_face_normals();
    let nodes = core::array::from_fn(|i| {
        let mut node = rig.add_mesh(meshes[i].clone(), Vec3::splat(scale));
        style_hex_node(&mut node);
        node
    });
    HexChunks {
        nodes,
        normals,
        detail: None,
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

fn push_gpu_mesh(
    coords: Vec<Vec3>,
    faces: Vec<[VertexIndex; 3]>,
    uvs: Vec<Vec2>,
    normals: Option<Vec<Vec3>>,
) -> Rc<RefCell<GpuMesh3d>> {
    Rc::new(RefCell::new(GpuMesh3d::new(
        coords,
        faces,
        normals,
        Some(uvs),
        false,
    )))
}

fn hex_fan_chunk_meshes(
    pipeline: &HexWorldPipelineRenderStruct,
    cell_face: &[u8],
) -> [Rc<RefCell<GpuMesh3d>>; CHUNK_COUNT] {
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
        let remap_vert = |v: u32,
                          coords: &mut [Vec<Vec3>],
                          uvs: &mut [Vec<Vec2>],
                          remap: &mut [i32]|
         -> VertexIndex {
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

    core::array::from_fn(|i| {
        push_gpu_mesh(
            core::mem::take(&mut coords[i]),
            core::mem::take(&mut faces[i]),
            core::mem::take(&mut uvs[i]),
            None,
        )
    })
}

fn hex_fan_cells_mesh(
    pipeline: &HexWorldPipelineRenderStruct,
    cells: &[i32],
) -> Rc<RefCell<GpuMesh3d>> {
    let verts = &pipeline.hex_verts;
    let tans = &pipeline.norm_height;
    let idx = &pipeline.tri_faces_idxs;
    let mut coords = Vec::with_capacity(cells.len() * 7);
    let mut uvs = Vec::with_capacity(cells.len() * 7);
    let mut faces = Vec::with_capacity(cells.len() * 6);

    for &cell_i in cells {
        let cell = cell_i as usize;
        let cnt = pipeline.neighbor_cnt[cell] as usize;
        let vert_base = coords.len() as u32;
        for local in 0..=cnt {
            let v = cell * 7 + local;
            let o = v * 3;
            coords.push(Vec3::new(
                verts[o] * DETAIL_SHELL,
                verts[o + 1] * DETAIL_SHELL,
                verts[o + 2] * DETAIL_SHELL,
            ));
            uvs.push(Vec2::new(tans[v], 0.5));
        }
        let origin = cell * 7;
        for t in 0..cnt {
            let base = cell * 18 + t * 3;
            let a = idx[base] as usize;
            let b = idx[base + 1] as usize;
            let c = idx[base + 2] as usize;
            if a == b && b == c {
                continue;
            }
            if a < origin || b < origin || c < origin {
                continue;
            }
            let la = a - origin;
            let lb = b - origin;
            let lc = c - origin;
            if la > cnt || lb > cnt || lc > cnt {
                continue;
            }
            faces.push([
                vert_base + la as u32,
                vert_base + lb as u32,
                vert_base + lc as u32,
            ]);
        }
    }

    push_gpu_mesh(coords, faces, uvs, None)
}

fn compact_chunk_meshes(
    pipeline: &HexWorldPipelineRenderStruct,
    cell_face: &[u8],
) -> [Rc<RefCell<GpuMesh3d>>; CHUNK_COUNT] {
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
                let tri = if outward < 0.0 {
                    [i0, b, a]
                } else {
                    [i0, a, b]
                };
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

fn empty_detail(n: usize) -> DetailLod {
    DetailLod {
        node: None,
        center: i32::MIN,
        seen: vec![0; n],
        seen_gen: 0,
        order: Vec::new(),
        dist: Vec::new(),
    }
}

fn sync_detail_lod(
    lod: &mut DetailLod,
    rig: &mut SceneNode3d,
    pipeline: &HexWorldPipelineRenderStruct,
    scale: f32,
) {
    let local = rig.inverse_rotation().mul_vec3(Vec3::Z);
    let cell = helpers::nearest_cell_idx(
        local.x,
        local.y,
        local.z,
        &pipeline.cell_unit_sphere_pos,
        pipeline.ws,
    );
    if cell == lod.center {
        return;
    }
    helpers::write_cell_disk(
        cell,
        DETAIL_HEX_RADIUS,
        &pipeline.neighbor_ids,
        &pipeline.neighbor_cnt,
        &mut lod.seen,
        &mut lod.seen_gen,
        &mut lod.order,
        &mut lod.dist,
    );
    if let Some(node) = lod.node.as_mut() {
        node.remove();
    }
    let mesh = hex_fan_cells_mesh(pipeline, &lod.order);
    let mut node = rig.add_mesh(mesh, Vec3::splat(scale));
    style_hex_node(&mut node);
    lod.node = Some(node);
    lod.center = cell;
}

fn spawn_hex_globe_view(
    rig: &mut SceneNode3d,
    pipeline: &HexWorldPipelineRenderStruct,
) -> HexChunks {
    let radius = globe_radius(pipeline);
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
    let mut chunks = spawn_chunk_nodes(rig, meshes, radius);
    if pipeline.ws > HEX_FAN_MAX_WS {
        chunks.detail = Some(empty_detail(pipeline.n as usize));
    }
    chunks
}

fn cull_hex_chunks(chunks: &mut HexChunks, eye: Vec3, rot: Quat) {
    let eye_len = eye.length();
    let keep = -0.35 * eye_len;
    for i in 0..CHUNK_COUNT {
        let n = chunks.normals[i];
        let wn = rot.mul_vec3(Vec3::new(n[0], n[1], n[2]));
        let facing = wn.x * eye.x + wn.y * eye.y + wn.z * eye.z;
        chunks.nodes[i].set_visible(facing > keep);
    }
}

fn ray_sphere(origin: Vec3, dir: Vec3, radius: f32) -> Option<Vec3> {
    let a = dir.dot(dir);
    let b = 2.0 * origin.dot(dir);
    let c = origin.dot(origin) - radius * radius;
    let disc = b * b - 4.0 * a * c;
    if disc < 0.0 || a == 0.0 {
        return None;
    }
    let root = disc.sqrt();
    let t0 = (-b - root) / (2.0 * a);
    let t1 = (-b + root) / (2.0 * a);
    let t = if t0 >= 0.0 { t0 } else { t1 };
    if t < 0.0 {
        return None;
    }
    Some(origin + dir * t)
}

fn hovered_hex(
    view: &LiveView,
    origin: Vec3,
    dir: Vec3,
    pipeline: &HexWorldPipelineRenderStruct,
) -> Option<i32> {
    if !matches!(view.kind, ActiveView::Hex(_)) {
        return None;
    }
    let hit = ray_sphere(origin, dir, globe_radius(pipeline))?;
    let local = view.rig.inverse_rotation().mul_vec3(hit);
    Some(helpers::nearest_cell_idx(
        local.x,
        local.y,
        local.z,
        &pipeline.cell_unit_sphere_pos,
        pipeline.ws,
    ))
}

fn hex_boundary(pipeline: &HexWorldPipelineRenderStruct, cell: i32, rot: Quat) -> Polyline3d {
    let cell = cell as usize;
    let cnt = pipeline.neighbor_cnt[cell] as usize;
    let verts = &pipeline.hex_verts;
    let radius = globe_radius(pipeline) * HOVER_EDGE_SHELL;
    let mut vertices = Vec::with_capacity(cnt + 1);
    for i in 0..cnt {
        let o = (cell * 7 + 1 + i) * 3;
        let corner = Vec3::new(verts[o], verts[o + 1], verts[o + 2]) * radius;
        vertices.push(rot.mul_vec3(corner));
    }
    if let Some(first) = vertices.first().copied() {
        vertices.push(first);
    }
    Polyline3d::new(vertices)
        .with_color(RED)
        .with_width(2.0)
        .with_depth_bias(0.004)
}

fn sync_hover(
    view: &mut LiveView,
    pick: Option<(Vec3, Vec3)>,
    pipeline: &HexWorldPipelineRenderStruct,
) {
    view.hover_cell = pick.and_then(|(origin, dir)| hovered_hex(view, origin, dir, pipeline));
}

fn sync_view(view: &mut LiveView, eye: Vec3, pipeline: &HexWorldPipelineRenderStruct) {
    let LiveView { rig, kind, .. } = view;
    if let ActiveView::Hex(chunks) = kind {
        cull_hex_chunks(chunks, eye, rig.rotation());
        if let Some(lod) = &mut chunks.detail {
            sync_detail_lod(lod, rig, pipeline, globe_radius(pipeline));
        }
    }
}

fn open_view(
    scene: &mut SceneNode3d,
    mode: RenderMode,
    pipeline: &HexWorldPipelineRenderStruct,
) -> LiveView {
    add_globe_lights(scene);
    let mut rig = scene.add_group();
    rig.set_position(Vec3::ZERO);
    let kind = match mode {
        RenderMode::PureSimplex => {
            spawn_pure_simplex_view(&mut rig, pipeline);
            scene
                .add_light(Light::point(6.0).with_intensity(0.45))
                .set_position(Vec3::new(-2.0, 1.5, 2.5));
            ActiveView::Simplex
        }
        RenderMode::HexGlobe => ActiveView::Hex(spawn_hex_globe_view(&mut rig, pipeline)),
    };
    LiveView {
        rig,
        kind,
        hover_cell: None,
    }
}

pub async fn render_hex_pipeline(pipeline: &HexWorldPipelineRenderStruct) {
    let mut window = Window::new("globe_render").await;
    window.set_ambient(0.8);
    let radius = globe_radius(pipeline);
    let view = Rc::new(RefCell::new(LiveView {
        rig: SceneNode3d::empty(),
        kind: ActiveView::Simplex,
        hover_cell: None,
    }));
    let mut camera = GlobeCamera::new(radius, view.clone(), pipeline);
    let mut scene = SceneNode3d::empty();
    let mut render_mode = RenderMode::HexGlobe;
    *view.borrow_mut() = open_view(&mut scene, render_mode, pipeline);
    let font = Font::default();

    let switch_render_mode = |curr_mode: RenderMode| {
        if curr_mode == RenderMode::PureSimplex {
            RenderMode::HexGlobe
        } else {
            RenderMode::PureSimplex
        }
    };

    while window.render_3d(&mut scene, &mut camera).await {
        for event in window.events().iter() {
            if let WindowEvent::Key(Key::G, Action::Press, _) = event.value {
                render_mode = switch_render_mode(render_mode);
                scene = SceneNode3d::empty();
                camera.frame_radius(radius);
                *view.borrow_mut() = open_view(&mut scene, render_mode, pipeline);
            }
        }
        if let Some(cell) = view.borrow().hover_cell {
            let rot = view.borrow().rig.rotation();
            let boundary = hex_boundary(pipeline, cell, rot);
            window.draw_polyline(&boundary);
            let label = cell.to_string();
            let advance = 0.56 * HOVER_TEXT_SCALE * label.len() as f32;
            let x = (window.width() as f32 - advance - 18.0).max(8.0);
            window.draw_text(&label, Vec2::new(x, 16.0), HOVER_TEXT_SCALE, &font, RED);
        }
    }
}
