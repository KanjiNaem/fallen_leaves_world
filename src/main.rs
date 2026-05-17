use fallen_leaves_world::{elevation_redistrib, img_gen, perlin_greyscale};

fn main() {
    let grid = perlin_greyscale::gen_octaved_perlin_greyscale(5000, 5000, 500, 8, 0.7);
    let terrace = elevation_redistrib::apply_terrace_redistrib(grid, 2.0, true);
    img_gen::gen_greyscale_img_from_vec(terrace);
}
