use fallen_leaves_world::{elevation_redistrib, img_gen, perlin_greyscale};

fn main() {
    let grid = perlin_greyscale::gen_octaved_perlin_greyscale(1000, 1000, 500, 8, 0.7);
    let _terrace = elevation_redistrib::apply_terrace_redistrib(&grid, 10.0, true);
    // img_gen::gen_greyscale_img_from_vec(&terrace);
    img_gen::gen_grey_with_waterlvl_highlighted(
        &grid,
        120.0,
        &img_gen::LandElevationPalette::default(),
    );
}
