// neighbor lookup
#[derive(Clone, Copy)]
pub struct IdxComponent {
    pub tile: i32,
    pub i: i32,
    pub j: i32,
}

pub fn build_idx_component(tile: i32, i: i32, j: i32) -> IdxComponent {
    IdxComponent { tile, i, j }
}

pub fn decode_idx(idx: i32, ws: i32) -> IdxComponent {
    let ws2 = ws * ws;
    let tile = idx / ws2;
    let rest = idx % ws2;
    let j = rest / ws;
    let i = rest % ws;
    IdxComponent { tile, i, j }
}

pub fn pack_idx(tile: i32, i: i32, j: i32, ws: i32) -> i32 {
    tile * ws * ws + j * ws + i
}

#[derive(Clone, Copy)]
pub enum Direction {
    N,
    NE,
    SE,
    S,
    SW,
    NW,
}

pub fn decode_dir(dir: Direction) -> IdxComponent {
    match dir {
        Direction::N => build_idx_component(0, 0, -1),
        Direction::NE => build_idx_component(0, 1, -1),
        Direction::SE => build_idx_component(0, 1, 0),
        Direction::S => build_idx_component(0, 0, 1),
        Direction::SW => build_idx_component(0, -1, 1),
        Direction::NW => build_idx_component(0, -1, 0),
    }
}

pub fn get_neighbor_idx(curr_idx: i32, ws: i32, dir: Direction) -> i32 {
    let ws2 = ws * ws;
    let north = 10 * ws2;
    let south = 10 * ws2 + 1;

    // poles not normal packs;; --> 5 neighbors are ring corners
    if curr_idx == north {
        let tile = match dir {
            Direction::N => 0,
            Direction::NE => 1,
            Direction::SE => 2,
            Direction::S => 3,
            Direction::SW => 4,
            Direction::NW => 0,
        };
        return pack_idx(tile, 0, 0, ws);
    }
    if curr_idx == south {
        let tile = match dir {
            Direction::N => 5,
            Direction::NE => 6,
            Direction::SE => 7,
            Direction::S => 8,
            Direction::SW => 9,
            Direction::NW => 5,
        };
        return pack_idx(tile, ws - 1, ws - 1, ws);
    }

    let idx_comp = decode_idx(curr_idx, ws);
    let step = decode_dir(dir);
    let tile = idx_comp.tile + step.tile; // step tile always 0, just here for completeness
    let i = idx_comp.i + step.i;
    let j = idx_comp.j + step.j;

    let in_range = |comp: i32| comp >= 0 && comp < ws;
    let east = |t: i32| 5 * (t / 5) + (t % 5 + 1) % 5;
    let west = |t: i32| 5 * (t / 5) + (t % 5 + 4) % 5;

    if in_range(i) && in_range(j) {
        return pack_idx(tile, i, j, ws);
    }

    // tile switch matching
    let (tile, i, j) = match (tile >= 5, i, j) {
        // north one axis
        (false, i, j) if i == ws && in_range(j) => (east(tile), 0, j),
        (false, i, j) if i == -1 && in_range(j) => (west(tile), ws - 1, j),
        (false, i, j) if in_range(i) && j == ws => (tile + 5, i, 0),
        (false, 0, -1) => return north,
        (false, i, -1) if in_range(i) => (west(tile), ws - 1, i),
        // noth both axes
        (false, i, j) if i == ws && j == ws => (east(tile) + 5, 0, 0),
        (false, i, -1) if i == ws => return north,
        (false, i, j) if i == -1 && j == ws => (west(tile) + 5, ws - 1, 0),
        (false, -1, -1) => (west(west(tile)), ws - 1, ws - 1),
        // south one axis
        (true, i, j) if i == ws && in_range(j) => (east(tile), 0, j),
        (true, i, j) if i == -1 && in_range(j) => (west(tile), ws - 1, j),
        (true, i, j) if i == ws - 1 && j == ws => return south,
        (true, i, j) if in_range(i) && j == ws => (east(tile), 0, i),
        (true, i, j) if in_range(i) && j == -1 => (tile - 5, i, ws - 1),
        // south both axes
        (true, i, j) if i == ws && j == ws => (east(east(tile)), 0, 0),
        (true, i, j) if i == ws && j == -1 => (east(tile) - 5, 0, ws - 1),
        (true, i, j) if i == -1 && j == ws => return south,
        (true, -1, -1) => (west(tile) - 5, ws - 1, ws - 1),

        _ => panic!("unhandled wrap tile={tile} i={i} j={j} n={ws}"), // keep panic for now, should never happen anyways amairait
    };

    pack_idx(tile, i, j, ws)
}

// unit vec to cell and inverse conversion
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

fn lerp_vec3(a: [f32; 3], b: [f32; 3], t: f32) -> [f32; 3] {
    add_vec3(scale_vec3(a, 1.0 - t), scale_vec3(b, t))
}


fn north_verts(k: i32) -> [f32; 3] {
    let a = std::f32::consts::TAU * (k.rem_euclid(5) as f32) / 5.0;
    [ICO_R * a.cos(), ICO_R * a.sin(), ICO_Z]
}

fn south_verts(k: i32) -> [f32; 3] {
    let a = std::f32::consts::TAU * (k.rem_euclid(5) as f32) / 5.0 + std::f32::consts::TAU / 10.0;
    [ICO_R * a.cos(), ICO_R * a.sin(), -ICO_Z]
}

// bilinear corners (A, B, D, C) for packed (u, v):: (0, 0) = A, (1, 0) = B, (0, 1) = D, (1, 1) = C.
// (0, 0) is poleward;; --> (0, 0) + N hits pole
fn rhombus_corners(tile: i32) -> [[f32; 3]; 4] {
    if tile < 5 {
        [NORTH_POLE, north_verts(tile + 1), north_verts(tile), south_verts(tile)]
    } else {
        let t = tile - 5;
        [north_verts(t), south_verts(t), south_verts(t + 4), SOUTH_POLE]
    }
}

fn bilinear_norm(corners: [[f32; 3]; 4], u: f32, v: f32) -> [f32; 3] {
    let [a, b, d, c] = corners;
    normalize_vec3(lerp_vec3(lerp_vec3(a, b, u), lerp_vec3(d, c, u), v))
}

pub fn cell_pos_on_sphere(idx: i32, ws: i32) -> [f32; 3] {
    let ws2 = ws * ws;
    if idx == 10 * ws2 {
        return NORTH_POLE;
    }
    if idx == 10 * ws2 + 1 {
        return SOUTH_POLE;
    }
    let comp = decode_idx(idx, ws);
    let n = ws as f32;
    let u = (comp.i as f32 + 0.5) / n;
    let v = (comp.j as f32 + 0.5) / n;
    bilinear_norm(rhombus_corners(comp.tile), u, v)
}

fn orient(a: [f32; 3], b: [f32; 3], c: [f32; 3]) -> ([f32; 3], [f32; 3], [f32; 3]) {
    if dot_vec3(c, cross_vec3(a, b)) >= 0.0 {
        (a, b, c)
    } else {
        (a, c, b)
    }
}

fn in_cone(point: [f32; 3], a: [f32; 3], b: [f32; 3], c: [f32; 3]) -> bool {
    let (a, b, c) = orient(a, b, c);
    dot_vec3(point, cross_vec3(a, b)) >= -EPS && dot_vec3(point, cross_vec3(b, c)) >= -EPS && dot_vec3(point, cross_vec3(c, a)) >= -EPS
}

fn barycentric(point: [f32; 3], a: [f32; 3], b: [f32; 3], c: [f32; 3]) -> [f32; 3] {
    let n = cross_vec3(sub_vec3(b, a), sub_vec3(c, a));
    let denom_t = dot_vec3(n, point);
    let hit = if denom_t.abs() < EPS {
        point
    } else {
        scale_vec3(point, dot_vec3(n, a) / denom_t)
    };
    let v0 = sub_vec3(b, a);
    let v1 = sub_vec3(c, a);
    let v2 = sub_vec3(hit, a);
    let d00 = dot_vec3(v0, v0);
    let d01 = dot_vec3(v0, v1);
    let d11 = dot_vec3(v1, v1);
    let d20 = dot_vec3(v2, v0);
    let d21 = dot_vec3(v2, v1);
    let denom = d00 * d11 - d01 * d01;
    if denom.abs() < EPS {
        return [1.0, 0.0, 0.0];
    }
    let v = (d11 * d20 - d01 * d21) / denom;
    let w = (d00 * d21 - d01 * d20) / denom;
    [1.0 - v - w, v, w]
}

fn clamp_ij_to_worldsize(i: i32, j: i32, ws: i32) -> (i32, i32) {
    (
        i.clamp(0, ws - 1),
        j.clamp(0, ws - 1),
    )
}

pub fn vec_to_cell_idx(x: f32, y: f32, z: f32, ws: i32) -> i32 {
    let point = normalize_vec3([x, y, z]);
    let ws2 = ws * ws;
    let n = ws as f32;
    let nearer_north = (0..5).all(|t| point[2] > dot_vec3(point, cell_pos_on_sphere(pack_idx(t, 0, 0, ws), ws)));
    if point[2] > 0.0 && nearer_north {
        return 10 * ws2;
    }
    let nearer_south = (5..10).all(|t| {
        -point[2] > dot_vec3(point, cell_pos_on_sphere(pack_idx(t, ws - 1, ws - 1, ws), ws))
    });
    if point[2] < 0.0 && nearer_south {
        return 10 * ws2 + 1;
    }

    let mut best = (0i32, 0.0f32, 0.0f32, f32::NEG_INFINITY);
    for tile in 0..10 {
        let [a, b, d, c] = rhombus_corners(tile);
        for (tri_u, tri_v, score) in [
            tri_uv(point, a, b, d, true),
            tri_uv(point, b, c, d, false),
        ] {
            if score > best.3 {
                best = (tile, tri_u, tri_v, score);
            }
        }
    }

    let (tile, u, v, _) = best;
    let (i, j) = clamp_ij_to_worldsize((u * n).floor() as i32, (v * n).floor() as i32, ws);
    pack_idx(tile, i, j, ws)
}

fn tri_uv(point: [f32; 3], a: [f32; 3], b: [f32; 3], c: [f32; 3], abd: bool) -> (f32, f32, f32) {
    let bary = barycentric(point, a, b, c);
    let score = if in_cone(point, a, b, c) {
        bary[0].min(bary[1]).min(bary[2])
    } else {
        bary[0].min(bary[1]).min(bary[2]) - 1.0
    };
    let (u, v) = if abd {
        (bary[1], bary[2])
    } else {
        (bary[0] + bary[1], bary[1] + bary[2])
    };
    (u, v, score)
}

