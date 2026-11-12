fn read_pos_xyz(pos: &[f32], idx: i32) -> [f32; 3] {
    let o = (idx * 3) as usize;
    [pos[o], pos[o + 1], pos[o + 2]]
}

fn tangent_basis(c: [f32; 3]) -> ([f32; 3], [f32; 3]) {
    let r = if c[2].abs() < 0.9 {
        [0.0, 0.0, 1.0]
    } else {
        [1.0, 0.0, 0.0]
    };
    let x = normalize_vec3(cross_vec3(r, c));
    let y = normalize_vec3(cross_vec3(c, x));
    (x, y)
}

// geodesic lattice: raw ids from barycentric face coords, remapped so poles sit at N-2 / N-1.
const ICO_Z: f32 = 0.4472135955; // 1/ sqrt(5);; --> z of northern ring verts
const ICO_R: f32 = 0.894427191; // 2/ sqrt(5);; xr rad of ring verts
const EPS: f32 = 1.0e-5; // mystery const for normalisation 
const NORTH_POLE: [f32; 3] = [0.0, 0.0, 1.0];
const SOUTH_POLE: [f32; 3] = [0.0, 0.0, -1.0];

fn add_vec3(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] + b[0], a[1] + b[1], a[2] + b[2]]
}

fn sub_vec3(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

fn scale_vec3(a: [f32; 3], s: f32) -> [f32; 3] {
    [a[0] * s, a[1] * s, a[2] * s]
}

fn dot_vec3(a: [f32; 3], b: [f32; 3]) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

fn cross_vec3(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

fn normalize_vec3(a: [f32; 3]) -> [f32; 3] {
    let vec_len = dot_vec3(a, a).sqrt();
    if vec_len < EPS {
        [0.0, 0.0, 1.0]
    } else {
        scale_vec3(a, 1.0 / vec_len)
    }
}

fn add3(a: [f32; 3], b: [f32; 3], c: [f32; 3]) -> [f32; 3] {
    add_vec3(add_vec3(a, b), c)
}

fn north_verts(k: i32) -> [f32; 3] {
    let a = std::f32::consts::TAU * k as f32 / 5.0;
    [ICO_R * a.cos(), ICO_R * a.sin(), ICO_Z]
}

fn south_verts(k: i32) -> [f32; 3] {
    let a = std::f32::consts::TAU * k as f32 / 5.0 + std::f32::consts::TAU / 10.0;
    [ICO_R * a.cos(), ICO_R * a.sin(), -ICO_Z]
}

fn ico_positions() -> [[f32; 3]; 12] {
    let mut pos_arr = [[0.0; 3]; 12];
    pos_arr[0] = NORTH_POLE;
    for i in 0..5 {
        pos_arr[1 + i] = north_verts(i as i32);
    }
    for i in 0..5 {
        pos_arr[6 + i] = south_verts(i as i32);
    }
    pos_arr[11] = SOUTH_POLE;
    pos_arr
}

fn ico_faces() -> [[usize; 3]; 20] {
    let mut faces = [[0usize; 3]; 20];
    for i in 0..5 {
        faces[i] = [0, 1 + i, 1 + ((i + 1) % 5)];
        faces[5 + i] = [1 + i, 1 + ((i + 1) % 5), 6 + i];
        faces[10 + i] = [1 + i, 6 + i, 6 + ((i + 4) % 5)];
        faces[15 + i] = [11, 6 + i, 6 + ((i + 4) % 5)];
    }
    faces
}

fn ico_edges() -> [(usize, usize); 30] {
    let faces = ico_faces();
    let mut edges = [(0usize, 0usize); 30];
    let mut cnt = 0;
    for curr_face in faces {
        for k in 0..3 {
            let mut a = curr_face[k];
            let mut b = curr_face[(k + 1) % 3];
            if a > b {
                core::mem::swap(&mut a, &mut b);
            }
            let mut found = false;
            for curr_edge in 0..cnt {
                if edges[curr_edge] == (a, b) {
                    found = true;
                    break;
                }
            }
            if !found {
                edges[cnt] = (a, b);
                cnt += 1;
            }
        }
    }
    debug_assert_eq!(cnt, 30);
    edges
}

fn edge_index(lo: usize, hi: usize, edges: &[(usize, usize); 30]) -> usize {
    match (0..30).position(|i| edges[i] == (lo, hi)) {
        Some(i) => i,
        None => panic!("unknown icosa edge ({lo},{hi})"),
    }
}

fn interior_index(a: i32, b: i32, n: i32) -> i32 {
    let mut idx = 0;
    for i in 1..a {
        idx += n - 1 - i;
    }
    idx + (b - 1)
}

fn interior_abc(local: i32, n: i32) -> (i32, i32, i32) {
    let mut idx = 0;
    let mut found = None;
    for a in 1..=n - 2 {
        let b_count = n - 1 - a;
        if local < idx + b_count {
            let b = 1 + (local - idx);
            found = Some((a, b, n - a - b));
            break;
        }
        idx += b_count;
    }
    match found {
        Some(abc) => abc,
        None => panic!("interior local {local} out of range for n={n}"),
    }
}

fn edge_raw(u: usize, v: usize, weight_u: i32, n: i32, edges: &[(usize, usize); 30]) -> i32 {
    let (lo, hi) = if u < v { (u, v) } else { (v, u) };
    let e = edge_index(lo, hi, edges) as i32;
    let t = if u == hi { weight_u } else { n - weight_u };
    12 + e * (n - 1) + (t - 1)
}

fn bary_to_raw(
    iv_idx0: usize,
    iv_idx1: usize,
    iv_idx2: usize,
    a: i32,
    b: i32,
    c: i32,
    n: i32,
    face_i: usize,
    edges: &[(usize, usize); 30],
) -> i32 {
    if a == n {
        return iv_idx0 as i32;
    }
    if b == n {
        return iv_idx1 as i32;
    }
    if c == n {
        return iv_idx2 as i32;
    }
    if c == 0 {
        return edge_raw(iv_idx0, iv_idx1, a, n, edges);
    }
    if b == 0 {
        return edge_raw(iv_idx0, iv_idx2, a, n, edges);
    }
    if a == 0 {
        return edge_raw(iv_idx1, iv_idx2, b, n, edges);
    }
    let interiors = (n - 1) * (n - 2) / 2;
    12 + 30 * (n - 1) + (face_i as i32) * interiors + interior_index(a, b, n)
}

fn raw_to_idx(raw: i32, n: i32) -> i32 {
    let v = 10 * n * n + 2;
    if raw == 0 {
        v - 2
    } else if raw == 11 {
        v - 1
    } else if raw < 11 {
        raw - 1
    } else {
        raw - 2
    }
}

fn idx_to_raw(idx: i32, ws: i32) -> i32 {
    let n = 10 * ws * ws + 2;
    if idx == n - 2 {
        0
    } else if idx == n - 1 {
        11
    } else if idx < 10 {
        idx + 1
    } else {
        idx + 2
    }
}

fn pos_from_raw(
    raw: i32,
    seg_freq: i32,
    ico_verts: &[[f32; 3]; 12],
    faces: &[[usize; 3]; 20],
    edges: &[(usize, usize); 30],
) -> [f32; 3] {
    if raw < 12 {
        return ico_verts[raw as usize];
    }

    let edge_hex_cnt = if seg_freq > 1 { 30 * (seg_freq - 1) } else { 0 };
    if raw < 12 + edge_hex_cnt {
        let ico_edge_idx = (raw - 12) / (seg_freq - 1);
        let edge_steps = (raw - 12) % (seg_freq - 1) + 1;
        let (v0, v1) = edges[ico_edge_idx as usize];
        return normalize_vec3(add_vec3(
            scale_vec3(ico_verts[v0], (seg_freq - edge_steps) as f32),
            scale_vec3(ico_verts[v1], edge_steps as f32),
        ));
    }

    let ico_face_hex_cnt = (seg_freq - 1) * (seg_freq - 2) / 2;
    let face_id = (raw - 12 - edge_hex_cnt) / ico_face_hex_cnt;
    let local = (raw - 12 - edge_hex_cnt) % ico_face_hex_cnt;
    let (a, b, c) = interior_abc(local, seg_freq);
    let [iv_idx0, iv_idx1, iv_idx2] = faces[face_id as usize];
    normalize_vec3(add3(
        scale_vec3(ico_verts[iv_idx0], a as f32),
        scale_vec3(ico_verts[iv_idx1], b as f32),
        scale_vec3(ico_verts[iv_idx2], c as f32),
    ))
}

fn push_neighbor(neighbor_ids: &mut [i32], neighbor_cnt: &mut [u8], idx_1: i32, idx_2: i32) {
    if idx_1 == idx_2 {
        return;
    }
    let base = (idx_1 * 6) as usize;
    let n1_ncnt = neighbor_cnt[idx_1 as usize] as usize;
    for i in 0..n1_ncnt {
        if neighbor_ids[base + i] == idx_2 {
            return;
        }
    }
    neighbor_ids[base + n1_ncnt] = idx_2;
    neighbor_cnt[idx_1 as usize] = (n1_ncnt + 1) as u8;
}

fn sort_ring_angular_soa(pos: &[f32], center_idx: i32, n_ids: &mut [i32]) {
    let center_pos = read_pos_xyz(pos, center_idx);
    let (tx, ty) = tangent_basis(center_pos);
    n_ids.sort_by(|&ia, &ib| {
        let da = sub_vec3(read_pos_xyz(pos, ia), center_pos);
        let db = sub_vec3(read_pos_xyz(pos, ib), center_pos);
        let aa = dot_vec3(da, ty).atan2(dot_vec3(da, tx));
        let ab = dot_vec3(db, ty).atan2(dot_vec3(db, tx));
        aa.partial_cmp(&ab).unwrap()
    });
}

pub fn cell_pos_on_sphere(idx: i32, ws: i32) -> [f32; 3] {
    let ico = ico_positions();
    let faces = ico_faces();
    let edges = ico_edges();
    pos_from_raw(idx_to_raw(idx, ws), ws, &ico, &faces, &edges)
}

fn ico_face_of_raw(
    raw: i32,
    ws: i32,
    faces: &[[usize; 3]; 20],
    edges: &[(usize, usize); 30],
) -> usize {
    if raw < 12 {
        let v = raw as usize;
        return faces
            .iter()
            .position(|f| f[0] == v || f[1] == v || f[2] == v)
            .unwrap();
    }
    let edge_n = if ws > 1 { 30 * (ws - 1) } else { 0 };
    if raw < 12 + edge_n {
        let e = (raw - 12) / (ws - 1);
        let (v0, v1) = edges[e as usize];
        return faces
            .iter()
            .position(|f| {
                (f[0] == v0 || f[1] == v0 || f[2] == v0) && (f[0] == v1 || f[1] == v1 || f[2] == v1)
            })
            .unwrap();
    }
    let interiors = (ws - 1) * (ws - 2) / 2;
    ((raw - 12 - edge_n) / interiors) as usize
}

pub fn ico_face_normals() -> [[f32; 3]; 20] {
    let verts = ico_positions();
    let faces = ico_faces();
    let mut normals = [[0.0; 3]; 20];
    for i in 0..20 {
        let a = verts[faces[i][0]];
        let b = verts[faces[i][1]];
        let c = verts[faces[i][2]];
        let mut n = cross_vec3(sub_vec3(b, a), sub_vec3(c, a));
        if dot_vec3(n, a) < 0.0 {
            n = scale_vec3(n, -1.0);
        }
        normals[i] = normalize_vec3(n);
    }
    normals
}

pub fn gen_cell_ico_face_data(face: &mut [u8], n: i32, ws: i32) {
    debug_assert_eq!(face.len(), n as usize);
    let faces = ico_faces();
    let edges = ico_edges();
    for idx in 0..n {
        face[idx as usize] = ico_face_of_raw(idx_to_raw(idx, ws), ws, &faces, &edges) as u8;
    }
}

pub fn gen_cell_unit_sphere_pos_data(data_arr: &mut [f32], n: i32, ws: i32) -> () {
    debug_assert_eq!(n, 10 * ws * ws + 2);
    debug_assert_eq!(data_arr.len(), (n * 3) as usize);
    let ico = ico_positions();
    let faces = ico_faces();
    let edges = ico_edges();
    for idx in 0..n {
        let pos = pos_from_raw(idx_to_raw(idx, ws), ws, &ico, &faces, &edges);
        let basis = (idx * 3) as usize;
        data_arr[basis] = pos[0];
        data_arr[basis + 1] = pos[1];
        data_arr[basis + 2] = pos[2];
    }
}

pub fn gen_neighbor_id_and_cnt_data(
    neighbor_ids: &mut [i32],
    neighbor_cnt: &mut [u8],
    unit_cell_pos: &[f32],
    n: i32,
    ws: i32,
) -> () {
    neighbor_ids.fill(-1);
    neighbor_cnt.fill(0);

    let faces = ico_faces();
    let edges = ico_edges();
    let steps = [
        (1, -1, 0),
        (-1, 1, 0),
        (1, 0, -1),
        (-1, 0, 1),
        (0, 1, -1),
        (0, -1, 1),
    ];

    for (face_i, face) in faces.iter().enumerate() {
        let (iv_idx0, iv_idx1, iv_idx2) = (face[0], face[1], face[2]);
        for a in 0..=ws {
            for b in 0..=(ws - a) {
                let c = ws - a - b;
                let id = raw_to_idx(
                    bary_to_raw(iv_idx0, iv_idx1, iv_idx2, a, b, c, ws, face_i, &edges),
                    ws,
                );
                for (dif_a, dif_b, dif_c) in steps {
                    let (a2, b2, c2) = (a + dif_a, b + dif_b, c + dif_c);
                    if a2 < 0 || b2 < 0 || c2 < 0 || a2 + b2 + c2 != ws {
                        continue;
                    }
                    let id2 = raw_to_idx(
                        bary_to_raw(iv_idx0, iv_idx1, iv_idx2, a2, b2, c2, ws, face_i, &edges),
                        ws,
                    );
                    push_neighbor(neighbor_ids, neighbor_cnt, id, id2);
                    push_neighbor(neighbor_ids, neighbor_cnt, id2, id);
                }
            }
        }
    }

    for idx in 0..n {
        let cnt = neighbor_cnt[idx as usize] as usize;
        debug_assert!(cnt <= 6);
        let base = (idx * 6) as usize;
        sort_ring_angular_soa(unit_cell_pos, idx, &mut neighbor_ids[base..base + cnt]);
    }
}

pub fn vec_to_cell_idx(x: f32, y: f32, z: f32, pos: &[f32], n: i32) -> i32 {
    debug_assert_eq!(pos.len(), (n * 3) as usize);
    let point = normalize_vec3([x, y, z]);
    let mut best = 0i32;
    let mut best_d = f32::MAX;
    for idx in 0..n {
        let p = read_pos_xyz(pos, idx);
        let d = sub_vec3(point, p);
        let d2 = dot_vec3(d, d);
        if d2 < best_d {
            best_d = d2;
            best = idx;
        }
    }
    best
}

fn bary_weights(c0: [f32; 3], c1: [f32; 3], c2: [f32; 3], p: [f32; 3]) -> Option<[f32; 3]> {
    let n0 = cross_vec3(c1, c2);
    let det = dot_vec3(c0, n0);
    if det.abs() < EPS {
        return None;
    }
    let inv = 1.0 / det;
    Some([
        dot_vec3(n0, p) * inv,
        dot_vec3(cross_vec3(c2, c0), p) * inv,
        dot_vec3(cross_vec3(c0, c1), p) * inv,
    ])
}

fn round_bary(a: f32, b: f32, c: f32, ws: i32) -> (i32, i32, i32) {
    let mut ints = [a.floor() as i32, b.floor() as i32, c.floor() as i32];
    let mut frac = [a - ints[0] as f32, b - ints[1] as f32, c - ints[2] as f32];
    let mut sum = ints[0] + ints[1] + ints[2];
    let mut guard = 0;
    while sum < ws && guard < 8 {
        let mut pick = 0;
        if frac[1] > frac[pick] {
            pick = 1;
        }
        if frac[2] > frac[pick] {
            pick = 2;
        }
        ints[pick] += 1;
        frac[pick] = 0.0;
        sum += 1;
        guard += 1;
    }
    guard = 0;
    while sum > ws && guard < 8 {
        let mut pick = None;
        for k in 0..3 {
            if ints[k] <= 0 {
                continue;
            }
            match pick {
                None => pick = Some(k),
                Some(p) if frac[k] < frac[p] => pick = Some(k),
                _ => {}
            }
        }
        let Some(pick) = pick else {
            break;
        };
        ints[pick] -= 1;
        frac[pick] = 1.0;
        sum -= 1;
        guard += 1;
    }
    (ints[0], ints[1], ints[2])
}

pub fn nearest_cell_idx(x: f32, y: f32, z: f32, pos: &[f32], ws: i32) -> i32 {
    let n = 10 * ws * ws + 2;
    debug_assert_eq!(pos.len(), (n * 3) as usize);
    let dir = normalize_vec3([x, y, z]);
    let ico = ico_positions();
    let faces = ico_faces();
    let edges = ico_edges();
    let steps = [
        (0, 0, 0),
        (1, -1, 0),
        (-1, 1, 0),
        (1, 0, -1),
        (-1, 0, 1),
        (0, 1, -1),
        (0, -1, 1),
    ];
    let mut best_idx = 0i32;
    let mut best_dot = f32::NEG_INFINITY;
    for (face_i, face) in faces.iter().enumerate() {
        let v0 = ico[face[0]];
        let v1 = ico[face[1]];
        let v2 = ico[face[2]];
        let Some(w) = bary_weights(v0, v1, v2, dir) else {
            continue;
        };
        if w[0] < -1.0e-3 || w[1] < -1.0e-3 || w[2] < -1.0e-3 {
            continue;
        }
        let sum = w[0].max(0.0) + w[1].max(0.0) + w[2].max(0.0);
        if sum < EPS {
            continue;
        }
        let scale = ws as f32 / sum;
        let (a, b, c) = round_bary(
            w[0].max(0.0) * scale,
            w[1].max(0.0) * scale,
            w[2].max(0.0) * scale,
            ws,
        );
        for (da, db, dc) in steps {
            let (a2, b2, c2) = (a + da, b + db, c + dc);
            if a2 < 0 || b2 < 0 || c2 < 0 || a2 + b2 + c2 != ws {
                continue;
            }
            let idx = raw_to_idx(
                bary_to_raw(face[0], face[1], face[2], a2, b2, c2, ws, face_i, &edges),
                ws,
            );
            if idx < 0 || idx >= n {
                continue;
            }
            let d = dot_vec3(dir, read_pos_xyz(pos, idx));
            if d > best_dot {
                best_dot = d;
                best_idx = idx;
            }
        }
    }
    best_idx
}

pub fn write_cell_disk(
    center: i32,
    radius: i32,
    neighbor_ids: &[i32],
    neighbor_cnt: &[u8],
    seen: &mut [u32],
    seen_gen: &mut u32,
    order: &mut Vec<i32>,
    dist: &mut Vec<u16>,
) {
    debug_assert!(radius >= 0);
    debug_assert_eq!(seen.len(), neighbor_cnt.len());
    debug_assert_eq!(neighbor_ids.len(), neighbor_cnt.len() * 6);
    order.clear();
    dist.clear();
    *seen_gen = seen_gen.wrapping_add(1);
    if *seen_gen == 0 {
        seen.fill(0);
        *seen_gen = 1;
    }
    let stamp = *seen_gen;
    seen[center as usize] = stamp;
    order.push(center);
    dist.push(0);
    let mut q = 0;
    while q < order.len() {
        let cur = order[q] as usize;
        let d = dist[q];
        q += 1;
        if d as i32 == radius {
            continue;
        }
        let base = cur * 6;
        let cnt = neighbor_cnt[cur] as usize;
        for k in 0..cnt {
            let nb = neighbor_ids[base + k];
            if nb < 0 {
                continue;
            }
            let nb_u = nb as usize;
            if seen[nb_u] == stamp {
                continue;
            }
            seen[nb_u] = stamp;
            order.push(nb);
            dist.push(d + 1);
        }
    }
}

pub fn gen_hex_vert_and_norm_height_and_tri_faces_data(
    global_data: &[f32],
    hex_vert_data_arr: &mut [f32],
    norm_height_data_arr: &mut [f32],
    tri_faces_data_array: &mut [u32],
    cell_unit_sphere_pos: &[f32],
    neighbor_ids: &[i32],
    neighbor_cnt: &[u8],
    n: i32,
    world_height: f32,
) -> () {
    let n = n as usize;
    let lo = 0.0;
    for curr_idx in 0..n {
        let height_idx = n + curr_idx;
        let curr_height = global_data[height_idx];
        let norm_height = ((curr_height - lo) / (world_height - lo)).clamp(0.0, 1.0);

        let total_vert_basis = curr_idx * 21;
        let uvs_basis = curr_idx * 7;
        let faces_basis = curr_idx * 18;
        let neighbor_basis = curr_idx * 6;
        let curr_neighbor_cnt = neighbor_cnt[curr_idx] as usize;
        let radius = 1.0;

        let center = read_pos_xyz(cell_unit_sphere_pos, curr_idx as i32);
        let scaled_center = scale_vec3(center, radius);
        hex_vert_data_arr[total_vert_basis] = scaled_center[0];
        hex_vert_data_arr[total_vert_basis + 1] = scaled_center[1];
        hex_vert_data_arr[total_vert_basis + 2] = scaled_center[2];
        norm_height_data_arr[uvs_basis] = norm_height;

        for i in 0..curr_neighbor_cnt {
            let neighbor_a = neighbor_ids[neighbor_basis + i] as usize;
            let neighbor_b = neighbor_ids[neighbor_basis + (i + 1) % curr_neighbor_cnt] as usize;
            let mut triple = [curr_idx, neighbor_a, neighbor_b];
            triple.sort_unstable();
            let corner = scale_vec3(
                normalize_vec3(add3(
                    read_pos_xyz(cell_unit_sphere_pos, triple[0] as i32),
                    read_pos_xyz(cell_unit_sphere_pos, triple[1] as i32),
                    read_pos_xyz(cell_unit_sphere_pos, triple[2] as i32),
                )),
                radius,
            );

            let adj_vert_basis = total_vert_basis + (i + 1) * 3;
            hex_vert_data_arr[adj_vert_basis] = corner[0];
            hex_vert_data_arr[adj_vert_basis + 1] = corner[1];
            hex_vert_data_arr[adj_vert_basis + 2] = corner[2];
            norm_height_data_arr[uvs_basis + 1 + i] = norm_height;
        }

        let hex_vert_basis = (curr_idx * 7) as u32;
        for i in 0..curr_neighbor_cnt {
            let mut i1 = hex_vert_basis + 1 + i as u32;
            let mut i2 = hex_vert_basis + 1 + ((i + 1) % curr_neighbor_cnt) as u32;
            let vert_0 = [
                hex_vert_data_arr[total_vert_basis],
                hex_vert_data_arr[total_vert_basis + 1],
                hex_vert_data_arr[total_vert_basis + 2],
            ];

            let tri_1_vert_basis = total_vert_basis + (i + 1) * 3;
            let tri_2_vert_basis = total_vert_basis + ((i + 1) % curr_neighbor_cnt + 1) * 3;
            let vert_1 = [
                hex_vert_data_arr[tri_1_vert_basis],
                hex_vert_data_arr[tri_1_vert_basis + 1],
                hex_vert_data_arr[tri_1_vert_basis + 2],
            ];
            let vert_2 = [
                hex_vert_data_arr[tri_2_vert_basis],
                hex_vert_data_arr[tri_2_vert_basis + 1],
                hex_vert_data_arr[tri_2_vert_basis + 2],
            ];

            if dot_vec3(
                cross_vec3(sub_vec3(vert_1, vert_0), sub_vec3(vert_2, vert_0)),
                center,
            ) < 0.0
            {
                core::mem::swap(&mut i1, &mut i2);
            }

            tri_faces_data_array[faces_basis + i * 3] = hex_vert_basis;
            tri_faces_data_array[faces_basis + i * 3 + 1] = i1;
            tri_faces_data_array[faces_basis + i * 3 + 2] = i2;
        }
    }
}
