use std::collections::VecDeque;

const CARDINAL_DELTAS: [(isize, isize); 4] = [(1, 0), (-1, 0), (0, 1), (0, -1)];

pub fn gen_water_body_size(
    width: usize,
    height: usize,
    terrain_map: &Vec<Vec<f64>>,
    water_lvl: f64,
) -> Vec<Vec<f64>> {
    let mut water_body_size_map = vec![vec![0.0; width]; height];
    let mut visited = vec![vec![false; width]; height];

    for y in 0..height {
        for x in 0..width {
            if terrain_map[y][x] > water_lvl || visited[y][x] {
                continue;
            }

            let mut queue = VecDeque::from([(x, y)]);
            let mut component = Vec::new();
            visited[y][x] = true;

            while let Some((cx, cy)) = queue.pop_front() {
                component.push((cx, cy));
                for &(dx, dy) in &CARDINAL_DELTAS {
                    let nx = cx as isize + dx;
                    let ny = cy as isize + dy;
                    if nx < 0 || ny < 0 {
                        continue;
                    }
                    let (nx, ny) = (nx as usize, ny as usize);
                    if nx >= width
                        || ny >= height
                        || terrain_map[ny][nx] > water_lvl
                        || visited[ny][nx]
                    {
                        continue;
                    }
                    visited[ny][nx] = true;
                    queue.push_back((nx, ny));
                }
            }

            let size = component.len() as f64;
            for (cx, cy) in component {
                water_body_size_map[cy][cx] = size;
            }
        }
    }

    water_body_size_map
}
