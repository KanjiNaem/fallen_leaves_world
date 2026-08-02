use fxhash::FxHashMap;

use crate::helpers;

pub fn assign_biome(
    width: usize,
    height: usize,
    terrain_map: Vec<Vec<f64>>,
    moisture_map: Vec<Vec<f64>>,
    rainfall_map: Vec<Vec<f64>>,
    magic_map: Vec<Vec<f64>>,
    chaos_map: Vec<Vec<f64>>,
    biome_set: BiomeParamPresetVals,
) -> Vec<Vec<f64>> {
    let height_diff_map: Vec<Vec<(f64, f64)>> =
        helpers::get_adj_height_diff_map(width, height, terrain_map);
    return vec![vec![0.0; width]; height];
}

pub enum BiomeParamPresetVals {
    Basic,
}
pub struct BiomeParamPresets {
    biome_params: fxhash::FxHashMap<
        String,
        (
            usize, // assignment prob (if determenistic: usize::MAX;; otherwise for n different biomes options: descision = prob_i / sum_of_nprob) (assumed to sum up to 100 for now)
            (i32, i32), // highest absolute height difference
            (i32, i32), // percent height (water lvl <-> top of world)
            (i32, i32), // percent moisture
            (i32, i32), // percent rainfall
            (i32, i32), // abs temp
            (i32, i32), // percent magic
            (i32, i32), // percent chaos
        ),
    >,
}

impl BiomeParamPresets {
    pub fn new(preset: &BiomeParamPresetVals) -> Self {
        let mut biome_params: FxHashMap<
            String,
            (
                usize,
                (i32, i32),
                (i32, i32),
                (i32, i32),
                (i32, i32),
                (i32, i32),
                (i32, i32),
                (i32, i32),
            ),
        > = FxHashMap::default();
        match preset {
            // (assignment prob, highest absolute height difference, percent height (water lvl <-> top of world), percent moisture, percent rainfall, abs temp, percent magic, percent chaos)
            BiomeParamPresetVals::Basic => {
                biome_params.insert(
                    format!("Grasslands"),
                    (
                        90,
                        (0, 15),
                        (0, 15),
                        (25, 40),
                        (15, 24),
                        (10, 30),
                        (0, 15),
                        (0, 15),
                    ),
                );
                biome_params.insert(
                    format!("Warped Grasslands"),
                    (
                        9,
                        (0, 15),
                        (0, 15),
                        (25, 40),
                        (15, 24),
                        (10, 30),
                        (16, 100),
                        (16, 100),
                    ),
                );
                biome_params.insert(
                    format!("Chaotic Grasslands"),
                    (
                        9,
                        (0, 15),
                        (0, 15),
                        (25, 40),
                        (15, 24),
                        (10, 30),
                        (0, 15),
                        (16, 100),
                    ),
                );
                biome_params.insert(
                    format!("Desert"),
                    (
                        usize::MAX,
                        (0, 15),
                        (0, 15),
                        (0, 15),
                        (0, 15),
                        (30, 60),
                        (0, 15),
                        (0, 15),
                    ),
                );
                biome_params.insert(
                    format!("Warped Desert"),
                    (
                        usize::MAX,
                        (0, 15),
                        (0, 15),
                        (0, 15),
                        (0, 15),
                        (30, 60),
                        (16, 100),
                        (16, 100),
                    ),
                );
                biome_params.insert(
                    format!("Chaotic Desert"),
                    (
                        usize::MAX,
                        (0, 15),
                        (0, 15),
                        (0, 15),
                        (0, 15),
                        (30, 60),
                        (0, 15),
                        (16, 100),
                    ),
                );
                biome_params.insert(
                    format!("Savannah"),
                    (
                        usize::MAX,
                        (0, 10),
                        (5, 15),
                        (25, 40),
                        (20, 40),
                        (31, 45),
                        (0, 15),
                        (0, 15),
                    ),
                );
                biome_params.insert(
                    format!("Warped Savannah"),
                    (
                        usize::MAX,
                        (0, 10),
                        (5, 15),
                        (25, 40),
                        (20, 40),
                        (31, 45),
                        (16, 100),
                        (16, 100),
                    ),
                );
                biome_params.insert(
                    format!("Chaotic Savannah"),
                    (
                        usize::MAX,
                        (0, 10),
                        (5, 15),
                        (25, 40),
                        (20, 40),
                        (31, 45),
                        (0, 15),
                        (16, 100),
                    ),
                );
                biome_params.insert(
                    format!("Steep Mountain"),
                    (
                        usize::MAX,
                        (60, i32::MAX),
                        (30, 100),
                        (0, 100),
                        (0, 100),
                        (0, 50),
                        (0, 15),
                        (0, 15),
                    ),
                );
                biome_params.insert(
                    format!("Warped Steep Mountain"),
                    (
                        usize::MAX,
                        (60, i32::MAX),
                        (30, 100),
                        (0, 100),
                        (0, 100),
                        (0, 50),
                        (16, 100),
                        (16, 100),
                    ),
                );
                biome_params.insert(
                    format!("Chaotic Steep Mountain"),
                    (
                        usize::MAX,
                        (60, i32::MAX),
                        (30, 100),
                        (0, 100),
                        (0, 100),
                        (0, 50),
                        (0, 15),
                        (16, 100),
                    ),
                );
                biome_params.insert(
                    format!("Mountain Peak"),
                    (
                        usize::MAX,
                        (0, 25),
                        (85, 100),
                        (0, 100),
                        (0, 100),
                        (0, 0),
                        (0, 15),
                        (0, 15),
                    ),
                );
                biome_params.insert(
                    format!("Warped Mountain Peak"),
                    (
                        usize::MAX,
                        (0, 25),
                        (85, 100),
                        (0, 100),
                        (0, 100),
                        (0, 0),
                        (16, 100),
                        (16, 100),
                    ),
                );
                biome_params.insert(
                    format!("Chaotic Mountain Peak"),
                    (
                        usize::MAX,
                        (0, 25),
                        (85, 100),
                        (0, 100),
                        (0, 100),
                        (0, 0),
                        (0, 15),
                        (16, 100),
                    ),
                );
                biome_params.insert(
                    format!("Ocean"),
                    (
                        usize::MAX,
                        (0, 0),
                        (-25, -5),
                        (0, 100),
                        (0, 100),
                        (0, 100),
                        (0, 15),
                        (0, 15),
                    ),
                );
                biome_params.insert(
                    format!("Warped Ocean"),
                    (
                        usize::MAX,
                        (0, 0),
                        (-25, -5),
                        (0, 100),
                        (0, 100),
                        (0, 100),
                        (16, 100),
                        (16, 100),
                    ),
                );
                biome_params.insert(
                    format!("Chaotic Ocean"),
                    (
                        usize::MAX,
                        (0, 0),
                        (-25, -5),
                        (0, 100),
                        (0, 100),
                        (0, 100),
                        (0, 15),
                        (16, 100),
                    ),
                );
                biome_params.insert(
                    format!("Deep Ocean"),
                    (
                        usize::MAX,
                        (0, 0),
                        (-100, -26),
                        (0, 100),
                        (0, 100),
                        (0, 100),
                        (0, 15),
                        (0, 15),
                    ),
                );
                biome_params.insert(
                    format!("Warped Deep Ocean"),
                    (
                        usize::MAX,
                        (0, 0),
                        (-100, -26),
                        (0, 100),
                        (0, 100),
                        (0, 100),
                        (16, 100),
                        (16, 100),
                    ),
                );
                biome_params.insert(
                    format!("Chaotic Deep Ocean"),
                    (
                        usize::MAX,
                        (0, 0),
                        (-100, -26),
                        (0, 100),
                        (0, 100),
                        (0, 100),
                        (0, 15),
                        (16, 100),
                    ),
                );
                biome_params.insert(
                    format!("Shallow Ocean"),
                    (
                        usize::MAX,
                        (0, 0),
                        (-4, -1),
                        (0, 100),
                        (0, 100),
                        (0, 100),
                        (0, 15),
                        (0, 15),
                    ),
                );
                biome_params.insert(
                    format!("Warped Shallow Ocean"),
                    (
                        usize::MAX,
                        (0, 0),
                        (-4, -1),
                        (0, 100),
                        (0, 100),
                        (0, 100),
                        (16, 100),
                        (16, 100),
                    ),
                );
                biome_params.insert(
                    format!("Chaotic Shallow Ocean"),
                    (
                        usize::MAX,
                        (0, 0),
                        (-4, -1),
                        (0, 100),
                        (0, 100),
                        (0, 100),
                        (0, 15),
                        (16, 100),
                    ),
                );
                biome_params.insert(
                    format!("Badlands"),
                    (
                        usize::MAX,
                        (16, 40),
                        (0, 50),
                        (5, 19),
                        (5, 15),
                        (30, 45),
                        (0, 15),
                        (0, 15),
                    ),
                );

                biome_params.insert(
                    format!("Warped Badlands"),
                    (
                        usize::MAX,
                        (16, 40),
                        (0, 50),
                        (5, 19),
                        (5, 15),
                        (30, 45),
                        (16, 100),
                        (16, 100),
                    ),
                );
                biome_params.insert(
                    format!("Chaotic Badlands"),
                    (
                        usize::MAX,
                        (16, 40),
                        (0, 50),
                        (5, 19),
                        (5, 15),
                        (30, 45),
                        (0, 15),
                        (16, 100),
                    ),
                );
                biome_params.insert(
                    format!("Forrest"),
                    (
                        usize::MAX,
                        (0, 40),
                        (0, 40),
                        (26, 35),
                        (25, 45),
                        (15, 30),
                        (0, 15),
                        (0, 15),
                    ),
                );
                biome_params.insert(
                    format!("Warped Forrest"),
                    (
                        usize::MAX,
                        (0, 40),
                        (0, 40),
                        (26, 35),
                        (25, 45),
                        (15, 30),
                        (16, 100),
                        (16, 100),
                    ),
                );
                biome_params.insert(
                    format!("Chaotic Forrest"),
                    (
                        usize::MAX,
                        (0, 40),
                        (0, 40),
                        (26, 35),
                        (25, 45),
                        (15, 30),
                        (0, 15),
                        (16, 100),
                    ),
                );
                biome_params.insert(
                    format!("Sparce Forrest"),
                    (
                        10,
                        (0, 40),
                        (0, 40),
                        (20, 25),
                        (15, 24),
                        (15, 30),
                        (0, 15),
                        (0, 15),
                    ),
                );
                biome_params.insert(
                    format!("Warped Sparce Forrest"),
                    (
                        1,
                        (0, 40),
                        (0, 40),
                        (20, 25),
                        (15, 24),
                        (15, 30),
                        (16, 100),
                        (16, 100),
                    ),
                );
                biome_params.insert(
                    format!("Chaotic Sparce Forrest"),
                    (
                        1,
                        (0, 40),
                        (0, 40),
                        (20, 25),
                        (15, 24),
                        (15, 30),
                        (0, 15),
                        (16, 100),
                    ),
                );
                biome_params.insert(
                    format!("Deep Forrest"),
                    (
                        usize::MAX,
                        (0, 40),
                        (0, 40),
                        (36, 40),
                        (25, 45),
                        (15, 30),
                        (0, 15),
                        (0, 15),
                    ),
                );
                biome_params.insert(
                    format!("Warped Deep Forrest"),
                    (
                        usize::MAX,
                        (0, 40),
                        (0, 40),
                        (36, 40),
                        (25, 45),
                        (15, 30),
                        (16, 100),
                        (16, 100),
                    ),
                );
                biome_params.insert(
                    format!("Chaotic Deep Forrest"),
                    (
                        usize::MAX,
                        (0, 40),
                        (0, 40),
                        (36, 40),
                        (25, 45),
                        (15, 30),
                        (0, 15),
                        (16, 100),
                    ),
                );
                biome_params.insert(
                    format!("Rainforrest"),
                    (
                        usize::MAX,
                        (0, 40),
                        (0, 15),
                        (41, 75),
                        (51, 75),
                        (31, 45),
                        (0, 15),
                        (0, 15),
                    ),
                );
                biome_params.insert(
                    format!("Warped Rainforrest"),
                    (
                        usize::MAX,
                        (0, 40),
                        (0, 15),
                        (41, 75),
                        (51, 75),
                        (31, 45),
                        (16, 100),
                        (16, 100),
                    ),
                );
                biome_params.insert(
                    format!("Chaotic Rainforrest"),
                    (
                        usize::MAX,
                        (0, 40),
                        (0, 15),
                        (41, 75),
                        (51, 75),
                        (31, 45),
                        (0, 15),
                        (16, 100),
                    ),
                );
                biome_params.insert(
                    format!("Swampy Forrest"),
                    (
                        usize::MAX,
                        (0, 15),
                        (0, 15),
                        (41, 75),
                        (51, 75),
                        (15, 30),
                        (0, 15),
                        (0, 15),
                    ),
                );
                biome_params.insert(
                    format!("Warped Swampy Forrest"),
                    (
                        usize::MAX,
                        (0, 15),
                        (0, 15),
                        (41, 75),
                        (51, 75),
                        (15, 30),
                        (16, 100),
                        (16, 100),
                    ),
                );
                biome_params.insert(
                    format!("Chaotic Swampy Forrest"),
                    (
                        usize::MAX,
                        (0, 15),
                        (0, 15),
                        (41, 75),
                        (51, 75),
                        (15, 30),
                        (0, 15),
                        (16, 100),
                    ),
                );
                biome_params.insert(
                    format!("Swamp"),
                    (
                        usize::MAX,
                        (0, 15),
                        (0, 15),
                        (41, 75),
                        (25, 50),
                        (15, 30),
                        (0, 15),
                        (0, 15),
                    ),
                );

                biome_params.insert(
                    format!("Warped Swamp"),
                    (
                        usize::MAX,
                        (0, 15),
                        (0, 15),
                        (41, 75),
                        (25, 50),
                        (15, 30),
                        (16, 100),
                        (16, 100),
                    ),
                );
                biome_params.insert(
                    format!("Chaotic Swamp"),
                    (
                        usize::MAX,
                        (0, 15),
                        (0, 15),
                        (41, 75),
                        (25, 50),
                        (15, 30),
                        (0, 15),
                        (16, 100),
                    ),
                );
                biome_params.insert(
                    format!("Ice Taiga"),
                    (
                        usize::MAX,
                        (0, 15),
                        (0, 40),
                        (26, 35),
                        (10, 24),
                        (-15, 0),
                        (0, 15),
                        (0, 15),
                    ),
                );
                biome_params.insert(
                    format!("Warped Ice Taiga"),
                    (
                        usize::MAX,
                        (0, 15),
                        (0, 40),
                        (26, 35),
                        (10, 24),
                        (-15, 0),
                        (16, 100),
                        (16, 100),
                    ),
                );
                biome_params.insert(
                    format!("Chaotic Ice Taiga"),
                    (
                        usize::MAX,
                        (0, 15),
                        (0, 40),
                        (26, 35),
                        (10, 24),
                        (-15, 0),
                        (0, 15),
                        (16, 100),
                    ),
                );
                biome_params.insert(
                    format!("Snowy Tundra"),
                    (
                        usize::MAX,
                        (0, 15),
                        (0, 40),
                        (20, 25),
                        (15, 24),
                        (-15, 0),
                        (0, 15),
                        (0, 15),
                    ),
                );
                biome_params.insert(
                    format!("Warped Snowy Tundra"),
                    (
                        usize::MAX,
                        (0, 15),
                        (0, 40),
                        (20, 25),
                        (15, 24),
                        (-15, 0),
                        (16, 100),
                        (16, 100),
                    ),
                );
                biome_params.insert(
                    format!("Chaotic Snowy Tundra"),
                    (
                        usize::MAX,
                        (0, 15),
                        (0, 40),
                        (20, 25),
                        (15, 24),
                        (-15, 0),
                        (0, 15),
                        (16, 100),
                    ),
                );
                biome_params.insert(
                    format!("Boreal Forrest"),
                    (
                        usize::MAX,
                        (0, 15),
                        (0, 40),
                        (26, 35),
                        (25, 45),
                        (-15, 0),
                        (0, 15),
                        (0, 15),
                    ),
                );
                biome_params.insert(
                    format!("Warped Boreal Forrest"),
                    (
                        usize::MAX,
                        (0, 15),
                        (0, 40),
                        (26, 35),
                        (25, 45),
                        (-15, 0),
                        (16, 100),
                        (16, 100),
                    ),
                );
                biome_params.insert(
                    format!("Chaotic Boreal Forrest"),
                    (
                        usize::MAX,
                        (0, 15),
                        (0, 40),
                        (26, 35),
                        (25, 45),
                        (-15, 0),
                        (0, 15),
                        (16, 100),
                    ),
                );
                biome_params.insert(
                    format!("Rocky Fields"),
                    (
                        usize::MAX,
                        (0, 25),
                        (16, 85),
                        (5, 24),
                        (5, 14),
                        (5, 15),
                        (0, 15),
                        (0, 15),
                    ),
                );
                biome_params.insert(
                    format!("Warped Rocky Fields"),
                    (
                        usize::MAX,
                        (0, 25),
                        (16, 85),
                        (5, 24),
                        (5, 14),
                        (5, 15),
                        (16, 100),
                        (16, 100),
                    ),
                );
                biome_params.insert(
                    format!("Chaotic Rocky Fields"),
                    (
                        usize::MAX,
                        (0, 25),
                        (16, 85),
                        (5, 24),
                        (5, 14),
                        (5, 15),
                        (0, 15),
                        (16, 100),
                    ),
                );
                Self {
                    biome_params: biome_params,
                }
            }
        }
    }
}
