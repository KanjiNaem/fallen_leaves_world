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
    let a = std::f32::consts::TAU * (k.rem_euclid(5) as f32) / 5.0;
    [ICO_R * a.cos(), ICO_R * a.sin(), ICO_Z]
}

fn south_verts(k: i32) -> [f32; 3] {
    let a = std::f32::consts::TAU * (k.rem_euclid(5) as f32) / 5.0 + std::f32::consts::TAU / 10.0;
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
        faces[i] = [0, 1 + i, 1 + (i + 1) % 5];
        faces[5 + i] = [1 + i, 1 + (i + 1) % 5, 6 + i];
        faces[10 + i] = [1 + i, 6 + i, 6 + (i + 4) % 5];
        faces[15 + i] = [11, 6 + i, 6 + (i + 4) % 5];
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
    i0: usize,
    i1: usize,
    i2: usize,
    a: i32,
    b: i32,
    c: i32,
    n: i32,
    face_i: usize,
    edges: &[(usize, usize); 30],
) -> i32 {
    if a == n {
        return i0 as i32;
    }
    if b == n {
        return i1 as i32;
    }
    if c == n {
        return i2 as i32;
    }
    if c == 0 {
        return edge_raw(i0, i1, a, n, edges);
    }
    if b == 0 {
        return edge_raw(i0, i2, a, n, edges);
    }
    if a == 0 {
        return edge_raw(i1, i2, b, n, edges);
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

    let edge_n = if seg_freq > 1 { 30 * (seg_freq - 1) } else { 0 };
    if raw < 12 + edge_n {
        let e = (raw - 12) / (seg_freq - 1);
        let steps = (raw - 12) % (seg_freq - 1) + 1;
        let (v0, v1) = edges[e as usize];
        return normalize_vec3(add_vec3(
            scale_vec3(ico_verts[v0], (seg_freq - steps) as f32),
            scale_vec3(ico_verts[v1], steps as f32),
        ));
    }

    let interiors = (seg_freq - 1) * (seg_freq - 2) / 2;
    let fi = (raw - 12 - edge_n) / interiors;
    let local = (raw - 12 - edge_n) % interiors;
    let (a, b, c) = interior_abc(local, seg_freq);
    let [i0, i1, i2] = faces[fi as usize];
    normalize_vec3(add3(
        scale_vec3(ico_verts[i0], a as f32),
        scale_vec3(ico_verts[i1], b as f32),
        scale_vec3(ico_verts[i2], c as f32),
    ))
}

fn push_neighbor(n_ids: &mut [i32], n_cnt: &mut [u8], n1_id: i32, n2_id: i32) {
    if n1_id == n2_id {
        return;
    }
    let base = (n1_id * 6) as usize;
    let n1_ncnt = n_cnt[n1_id as usize] as usize;
    for i in 0..n1_ncnt {
        if n_ids[base + i] == n2_id {
            return;
        }
    }
    n_ids[base + n1_ncnt] = n2_id;
    n_cnt[n1_id as usize] = (n1_ncnt + 1) as u8;
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

// TODO: make this one more readable i hate using single letter vars
pub fn gen_neighbor_id_and_cnt_data(
    id_data_arr: &mut [i32],
    cnt_data_arr: &mut [u8],
    pos: &[f32],
    n: i32,
    ws: i32,
) -> () {
    // debug_assert_eq!(n, 10 * ws * ws + 2);
    id_data_arr.fill(-1);
    cnt_data_arr.fill(0);

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
        let (i0, i1, i2) = (face[0], face[1], face[2]);
        for a in 0..=ws {
            for b in 0..=(ws - a) {
                let c = ws - a - b;
                let id = raw_to_idx(bary_to_raw(i0, i1, i2, a, b, c, ws, face_i, &edges), ws);
                for (da, db, dc) in steps {
                    let (a2, b2, c2) = (a + da, b + db, c + dc);
                    if a2 < 0 || b2 < 0 || c2 < 0 || a2 + b2 + c2 != ws {
                        continue;
                    }
                    let id2 = raw_to_idx(bary_to_raw(i0, i1, i2, a2, b2, c2, ws, face_i, &edges), ws);
                    push_neighbor(id_data_arr, cnt_data_arr, id, id2);
                    push_neighbor(id_data_arr, cnt_data_arr, id2, id);
                }
            }
        }
    }

    for idx in 0..n {
        let cnt = cnt_data_arr[idx as usize] as usize;
        debug_assert!(cnt <= 6);
        let base = (idx * 6) as usize;
        sort_ring_angular_soa(pos, idx, &mut id_data_arr[base..base + cnt]);
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

pub fn gen_hex_vert_and_norm_tan_and_tri_faces_data(global_data: &[f32], hex_vert_data_arr: &mut [f32], norm_tan_data_arr: &mut [f32], tri_faces_data_array: &mut [u32], cell_unit_sphere_pos: &[f32], neighbor_ids: &[i32], neighbor_cnt: &[u8], n: i32, ws: i32, world_height: f32) -> () {
    let n = n as usize;
    let lo = 0.0;
    for curr_idx in 0..n {
        let height_idx = n + curr_idx;
        let curr_height = global_data[height_idx];
        let tangent = ((curr_height - lo) / (world_height - lo)).clamp(0.0, 1.0);
        
        let total_vert_basis = curr_idx * 21;
        let uvs_basis = curr_idx * 7; 
        let faces_basis = curr_idx * 18;
        let neighbor_basis = curr_idx * 6;
        let curr_neighbor_cnt = neighbor_cnt[curr_idx] as usize;
        let radius = 1.0;
        let _ = ws;

        let center = read_pos_xyz(cell_unit_sphere_pos, curr_idx as i32);
        let scaled_center = scale_vec3(center, radius);
        hex_vert_data_arr[total_vert_basis] = scaled_center[0];
        hex_vert_data_arr[total_vert_basis + 1] = scaled_center[1];
        hex_vert_data_arr[total_vert_basis + 2] = scaled_center[2];
        norm_tan_data_arr[uvs_basis] = tangent;

        for i in 0..curr_neighbor_cnt {
            let n_a = neighbor_ids[neighbor_basis + i] as usize;
            let n_b = neighbor_ids[neighbor_basis + (i + 1) % curr_neighbor_cnt] as usize;
            let mut triple = [curr_idx, n_a, n_b];
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
            norm_tan_data_arr[uvs_basis + 1 + i] = tangent;
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

            if dot_vec3(cross_vec3(sub_vec3(vert_1, vert_0), sub_vec3(vert_2, vert_0)), center) < 0.0 {
                core::mem::swap(&mut i1, &mut i2);
            }

            tri_faces_data_array[faces_basis + i * 3] = hex_vert_basis;
            tri_faces_data_array[faces_basis + i * 3 + 1] = i1;
            tri_faces_data_array[faces_basis + i * 3 + 2] = i2;
        }

    }
}
