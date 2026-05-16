use fallen_leaves_world::{perlin_greyscale, img_gen};

fn main() {
    let grid = perlin_greyscale::gen_octaved_perlin_greyscale(5000, 5000, 500, 8, 0.7);
    img_gen::gen_greyscale_img_from_vec(grid);
}
