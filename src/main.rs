use fallen_leaves_world::{perlin_greyscale, img_gen};

fn main() {
    let grid = perlin_greyscale::gen_octaved_perlin_greyscale(1000, 1000, 100, 8, 0.5);
    // for row in &grid {
    //     println!(
    //         "{}",
    //         row.iter()
    //             .map(|n| n.to_string())
    //             .collect::<Vec<_>>()
    //             .join(" ")
    //     );
    // }
    img_gen::gen_greyscale_img_from_vec(grid);
}
