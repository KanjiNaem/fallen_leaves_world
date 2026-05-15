use fallen_leaves_world::perlin_greyscale;

fn main() {
    let grid = perlin_greyscale::gen_perlin_greyscale(1000, 1000, 100);
    for row in &grid {
        println!(
            "{}",
            row.iter()
                .map(|n| n.to_string())
                .collect::<Vec<_>>()
                .join(" ")
        );
    }
}
