use fxhash::FxHashMap;

use crate::{helpers, moisture_map};

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
            f64,
            (f64, f64),
            (f64, f64),
            (f64, f64),
            (f64, f64),
            (f64, f64),
            (f64, f64),
        ),
    >,
}

impl BiomeParamPresets {
    pub fn new(preset: &BiomeParamPresetVals) -> Self {
        let mut biome_params: FxHashMap<
            String,
            (
                f64,
                (f64, f64),
                (f64, f64),
                (f64, f64),
                (f64, f64),
                (f64, f64),
                (f64, f64),
            ),
        > = FxHashMap::default();
        match preset {
            // (absolute height difference, percent height (water lvl <-> top of world), percent moisture, percent rainfall, abs temp, percent magic, percent chaos)
            BiomeParamPresetVals::Basic => {
                biome_params.insert(
                    format!("Grasslands"),
                    (
                        15.0,
                        (0.0, 15.0),
                        (25.0, 40.0),
                        (30.0, 50.0),
                        (0.0, 0.0),
                        (0.0, 15.0),
                        (0.0, 15.0),
                    ),
                );
                biome_params.insert(
                    format!("Warped Grasslands"),
                    (
                        15.0,
                        (0.0, 15.0),
                        (25.0, 40.0),
                        (30.0, 50.0),
                        (0.0, 0.0),
                        (15.0, 100.0),
                        (15.0, 100.0),
                    ),
                );
                biome_params.insert(
                    format!("Chaotic Grasslands"),
                    (
                        15.0,
                        (0.0, 15.0),
                        (25.0, 40.0),
                        (30.0, 50.0),
                        (0.0, 0.0),
                        (0.0, 15.0),
                        (15.0, 100.0),
                    ),
                );
                biome_params.insert(
                    format!("Desert"),
                    (
                        15.0,
                        (0.0, 15.0),
                        (0.0, 15.0),
                        (0.0, 15.0),
                        (0.0, 0.0),
                        (0.0, 15.0),
                        (0.0, 15.0),
                    ),
                );
                biome_params.insert(
                    format!("Warped Desert"),
                    (
                        15.0,
                        (0.0, 15.0),
                        (0.0, 15.0),
                        (0.0, 15.0),
                        (0.0, 0.0),
                        (15.0, 100.0),
                        (15.0, 100.0),
                    ),
                );
                biome_params.insert(
                    format!("Chaotic Desert"),
                    (
                        15.0,
                        (0.0, 15.0),
                        (0.0, 15.0),
                        (0.0, 15.0),
                        (0.0, 0.0),
                        (0.0, 15.0),
                        (15.0, 100.0),
                    ),
                );
                // (absolute height difference, percent height (water lvl <-> top of world), percent moisture, percent rainfall, percent magic, percent chaos)
                biome_params.insert(
                    format!("Savannah"),
                    (
                        15.0,
                        (0.0, 15.0),
                        (25.0, 40.0),
                        (30.0, 50.0),
                        (0.0, 0.0),
                        (0.0, 15.0),
                        (0.0, 15.0),
                    ),
                );
                biome_params.insert(
                    format!("Warped Savannah"),
                    (
                        15.0,
                        (0.0, 15.0),
                        (25.0, 40.0),
                        (30.0, 50.0),
                        (0.0, 0.0),
                        (15.0, 100.0),
                        (15.0, 100.0),
                    ),
                );
                biome_params.insert(
                    format!("Chaotic Savannah"),
                    (
                        15.0,
                        (0.0, 15.0),
                        (25.0, 40.0),
                        (30.0, 50.0),
                        (0.0, 0.0),
                        (0.0, 15.0),
                        (15.0, 100.0),
                    ),
                );
                biome_params.insert(
                    format!("Steep Mountain"),
                    (
                        15.0,
                        (0.0, 15.0),
                        (25.0, 40.0),
                        (30.0, 50.0),
                        (0.0, 0.0),
                        (0.0, 15.0),
                        (0.0, 15.0),
                    ),
                );
                biome_params.insert(
                    format!("Warped Steep Mountain"),
                    (
                        15.0,
                        (0.0, 15.0),
                        (25.0, 40.0),
                        (30.0, 50.0),
                        (0.0, 0.0),
                        (15.0, 100.0),
                        (15.0, 100.0),
                    ),
                );
                biome_params.insert(
                    format!("Chaotic Steep Mountain"),
                    (
                        15.0,
                        (0.0, 15.0),
                        (25.0, 40.0),
                        (30.0, 50.0),
                        (0.0, 0.0),
                        (0.0, 15.0),
                        (15.0, 100.0),
                    ),
                );
                biome_params.insert(
                    format!("Mountain Peak"),
                    (
                        15.0,
                        (0.0, 15.0),
                        (25.0, 40.0),
                        (30.0, 50.0),
                        (0.0, 0.0),
                        (0.0, 15.0),
                        (0.0, 15.0),
                    ),
                );
                biome_params.insert(
                    format!("Warped Mountain Peak"),
                    (
                        15.0,
                        (0.0, 15.0),
                        (25.0, 40.0),
                        (30.0, 50.0),
                        (0.0, 0.0),
                        (15.0, 100.0),
                        (15.0, 100.0),
                    ),
                );
                biome_params.insert(
                    format!("Chaotic Mountain Peak"),
                    (
                        15.0,
                        (0.0, 15.0),
                        (25.0, 40.0),
                        (30.0, 50.0),
                        (0.0, 0.0),
                        (0.0, 15.0),
                        (15.0, 100.0),
                    ),
                );
                biome_params.insert(
                    format!("Ocean"),
                    (
                        15.0,
                        (0.0, 15.0),
                        (25.0, 40.0),
                        (30.0, 50.0),
                        (0.0, 0.0),
                        (0.0, 15.0),
                        (0.0, 15.0),
                    ),
                );
                biome_params.insert(
                    format!("Warped Ocean"),
                    (
                        15.0,
                        (0.0, 15.0),
                        (25.0, 40.0),
                        (30.0, 50.0),
                        (0.0, 0.0),
                        (15.0, 100.0),
                        (15.0, 100.0),
                    ),
                );
                biome_params.insert(
                    format!("Chaotic Ocean"),
                    (
                        15.0,
                        (0.0, 15.0),
                        (25.0, 40.0),
                        (30.0, 50.0),
                        (0.0, 0.0),
                        (0.0, 15.0),
                        (15.0, 100.0),
                    ),
                );
                biome_params.insert(
                    format!("Deep Ocean"),
                    (
                        15.0,
                        (0.0, 15.0),
                        (25.0, 40.0),
                        (30.0, 50.0),
                        (0.0, 0.0),
                        (0.0, 15.0),
                        (0.0, 15.0),
                    ),
                );
                biome_params.insert(
                    format!("Warped Deep Ocean"),
                    (
                        15.0,
                        (0.0, 15.0),
                        (25.0, 40.0),
                        (30.0, 50.0),
                        (0.0, 0.0),
                        (15.0, 100.0),
                        (15.0, 100.0),
                    ),
                );
                biome_params.insert(
                    format!("Chaotic Deep Ocean"),
                    (
                        15.0,
                        (0.0, 15.0),
                        (25.0, 40.0),
                        (30.0, 50.0),
                        (0.0, 0.0),
                        (0.0, 15.0),
                        (15.0, 100.0),
                    ),
                );
                biome_params.insert(
                    format!("Shallow Ocean"),
                    (
                        15.0,
                        (0.0, 15.0),
                        (25.0, 40.0),
                        (30.0, 50.0),
                        (0.0, 0.0),
                        (0.0, 15.0),
                        (0.0, 15.0),
                    ),
                );
                biome_params.insert(
                    format!("Warped Shallow Ocean"),
                    (
                        15.0,
                        (0.0, 15.0),
                        (25.0, 40.0),
                        (30.0, 50.0),
                        (0.0, 0.0),
                        (15.0, 100.0),
                        (15.0, 100.0),
                    ),
                );
                biome_params.insert(
                    format!("Chaotic Shallow Ocean"),
                    (
                        15.0,
                        (0.0, 15.0),
                        (25.0, 40.0),
                        (30.0, 50.0),
                        (0.0, 0.0),
                        (0.0, 15.0),
                        (15.0, 100.0),
                    ),
                );
                biome_params.insert(
                    format!("Badlands"),
                    (
                        15.0,
                        (0.0, 15.0),
                        (25.0, 40.0),
                        (30.0, 50.0),
                        (0.0, 0.0),
                        (0.0, 15.0),
                        (0.0, 15.0),
                    ),
                );
                biome_params.insert(
                    format!("Warped Badlands"),
                    (
                        15.0,
                        (0.0, 15.0),
                        (25.0, 40.0),
                        (30.0, 50.0),
                        (0.0, 0.0),
                        (15.0, 100.0),
                        (15.0, 100.0),
                    ),
                );
                biome_params.insert(
                    format!("Chaotic Badlands"),
                    (
                        15.0,
                        (0.0, 15.0),
                        (25.0, 40.0),
                        (30.0, 50.0),
                        (0.0, 0.0),
                        (0.0, 15.0),
                        (15.0, 100.0),
                    ),
                );
                biome_params.insert(
                    format!("Forrest"),
                    (
                        15.0,
                        (0.0, 15.0),
                        (25.0, 40.0),
                        (30.0, 50.0),
                        (0.0, 0.0),
                        (0.0, 15.0),
                        (0.0, 15.0),
                    ),
                );
                biome_params.insert(
                    format!("Warped Forrest"),
                    (
                        15.0,
                        (0.0, 15.0),
                        (25.0, 40.0),
                        (30.0, 50.0),
                        (0.0, 0.0),
                        (15.0, 100.0),
                        (15.0, 100.0),
                    ),
                );
                biome_params.insert(
                    format!("Chaotic Forrest"),
                    (
                        15.0,
                        (0.0, 15.0),
                        (25.0, 40.0),
                        (30.0, 50.0),
                        (0.0, 0.0),
                        (0.0, 15.0),
                        (15.0, 100.0),
                    ),
                );
                biome_params.insert(
                    format!("Sparce Forrest"),
                    (
                        15.0,
                        (0.0, 15.0),
                        (25.0, 40.0),
                        (30.0, 50.0),
                        (0.0, 0.0),
                        (0.0, 15.0),
                        (0.0, 15.0),
                    ),
                );
                biome_params.insert(
                    format!("Warped Sparce Forrest"),
                    (
                        15.0,
                        (0.0, 15.0),
                        (25.0, 40.0),
                        (30.0, 50.0),
                        (0.0, 0.0),
                        (15.0, 100.0),
                        (15.0, 100.0),
                    ),
                );
                biome_params.insert(
                    format!("Deep Forrest"),
                    (
                        15.0,
                        (0.0, 15.0),
                        (25.0, 40.0),
                        (30.0, 50.0),
                        (0.0, 0.0),
                        (0.0, 15.0),
                        (0.0, 15.0),
                    ),
                );
                biome_params.insert(
                    format!("Warped Deep Forrest"),
                    (
                        15.0,
                        (0.0, 15.0),
                        (25.0, 40.0),
                        (30.0, 50.0),
                        (0.0, 0.0),
                        (15.0, 100.0),
                        (15.0, 100.0),
                    ),
                );
                biome_params.insert(
                    format!("Chaotic Deep Forrest"),
                    (
                        15.0,
                        (0.0, 15.0),
                        (25.0, 40.0),
                        (30.0, 50.0),
                        (0.0, 0.0),
                        (0.0, 15.0),
                        (15.0, 100.0),
                    ),
                );
                biome_params.insert(
                    format!("Rainforrest"),
                    (
                        15.0,
                        (0.0, 15.0),
                        (25.0, 40.0),
                        (30.0, 50.0),
                        (0.0, 0.0),
                        (0.0, 15.0),
                        (0.0, 15.0),
                    ),
                );
                biome_params.insert(
                    format!("Warped Rainforrest"),
                    (
                        15.0,
                        (0.0, 15.0),
                        (25.0, 40.0),
                        (30.0, 50.0),
                        (0.0, 0.0),
                        (15.0, 100.0),
                        (15.0, 100.0),
                    ),
                );
                biome_params.insert(
                    format!("Chaotic Rainforrest"),
                    (
                        15.0,
                        (0.0, 15.0),
                        (25.0, 40.0),
                        (30.0, 50.0),
                        (0.0, 0.0),
                        (0.0, 15.0),
                        (15.0, 100.0),
                    ),
                );
                biome_params.insert(
                    format!("Swampy Forrest"),
                    (
                        15.0,
                        (0.0, 15.0),
                        (25.0, 40.0),
                        (30.0, 50.0),
                        (0.0, 0.0),
                        (0.0, 15.0),
                        (0.0, 15.0),
                    ),
                );
                biome_params.insert(
                    format!("Warped Swampy Forrest"),
                    (
                        15.0,
                        (0.0, 15.0),
                        (25.0, 40.0),
                        (30.0, 50.0),
                        (0.0, 0.0),
                        (15.0, 100.0),
                        (15.0, 100.0),
                    ),
                );
                biome_params.insert(
                    format!("Chaotic Swampy Forrest"),
                    (
                        15.0,
                        (0.0, 15.0),
                        (25.0, 40.0),
                        (30.0, 50.0),
                        (0.0, 0.0),
                        (0.0, 15.0),
                        (15.0, 100.0),
                    ),
                );
                biome_params.insert(
                    format!("Swamp"),
                    (
                        15.0,
                        (0.0, 15.0),
                        (25.0, 40.0),
                        (30.0, 50.0),
                        (0.0, 0.0),
                        (0.0, 15.0),
                        (0.0, 15.0),
                    ),
                );
                biome_params.insert(
                    format!("Warped Swamp"),
                    (
                        15.0,
                        (0.0, 15.0),
                        (25.0, 40.0),
                        (30.0, 50.0),
                        (0.0, 0.0),
                        (15.0, 100.0),
                        (15.0, 100.0),
                    ),
                );
                biome_params.insert(
                    format!("Chaotic Swamp"),
                    (
                        15.0,
                        (0.0, 15.0),
                        (25.0, 40.0),
                        (30.0, 50.0),
                        (0.0, 0.0),
                        (0.0, 15.0),
                        (15.0, 100.0),
                    ),
                );
                biome_params.insert(
                    format!("Marsh"),
                    (
                        15.0,
                        (0.0, 15.0),
                        (25.0, 40.0),
                        (30.0, 50.0),
                        (0.0, 0.0),
                        (0.0, 15.0),
                        (0.0, 15.0),
                    ),
                );
                biome_params.insert(
                    format!("Warped March"),
                    (
                        15.0,
                        (0.0, 15.0),
                        (25.0, 40.0),
                        (30.0, 50.0),
                        (0.0, 0.0),
                        (15.0, 100.0),
                        (15.0, 100.0),
                    ),
                );
                biome_params.insert(
                    format!("Chaotic Marsh"),
                    (
                        15.0,
                        (0.0, 15.0),
                        (25.0, 40.0),
                        (30.0, 50.0),
                        (0.0, 0.0),
                        (0.0, 15.0),
                        (15.0, 100.0),
                    ),
                );
                biome_params.insert(
                    format!("Ice Taiga"),
                    (
                        15.0,
                        (0.0, 15.0),
                        (25.0, 40.0),
                        (30.0, 50.0),
                        (0.0, 0.0),
                        (0.0, 15.0),
                        (0.0, 15.0),
                    ),
                );
                biome_params.insert(
                    format!("Warped Ice Taiga"),
                    (
                        15.0,
                        (0.0, 15.0),
                        (25.0, 40.0),
                        (30.0, 50.0),
                        (0.0, 0.0),
                        (15.0, 100.0),
                        (15.0, 100.0),
                    ),
                );
                biome_params.insert(
                    format!("Chaotic Ice Taiga"),
                    (
                        15.0,
                        (0.0, 15.0),
                        (25.0, 40.0),
                        (30.0, 50.0),
                        (0.0, 0.0),
                        (0.0, 15.0),
                        (15.0, 100.0),
                    ),
                );
                biome_params.insert(
                    format!("Snowy Tundra"),
                    (
                        15.0,
                        (0.0, 15.0),
                        (25.0, 40.0),
                        (30.0, 50.0),
                        (0.0, 0.0),
                        (0.0, 15.0),
                        (0.0, 15.0),
                    ),
                );
                biome_params.insert(
                    format!("Warped Snowy Tundra"),
                    (
                        15.0,
                        (0.0, 15.0),
                        (25.0, 40.0),
                        (30.0, 50.0),
                        (0.0, 0.0),
                        (15.0, 100.0),
                        (15.0, 100.0),
                    ),
                );
                biome_params.insert(
                    format!("Chaotic Snowy Tundra"),
                    (
                        15.0,
                        (0.0, 15.0),
                        (25.0, 40.0),
                        (30.0, 50.0),
                        (0.0, 0.0),
                        (0.0, 15.0),
                        (15.0, 100.0),
                    ),
                );
                biome_params.insert(
                    format!("Boreal Forrest"),
                    (
                        15.0,
                        (0.0, 15.0),
                        (25.0, 40.0),
                        (30.0, 50.0),
                        (0.0, 0.0),
                        (0.0, 15.0),
                        (0.0, 15.0),
                    ),
                );
                biome_params.insert(
                    format!("Warped Boreal Forrest"),
                    (
                        15.0,
                        (0.0, 15.0),
                        (25.0, 40.0),
                        (30.0, 50.0),
                        (0.0, 0.0),
                        (15.0, 100.0),
                        (15.0, 100.0),
                    ),
                );
                biome_params.insert(
                    format!("Chaotic Boreal Forrest"),
                    (
                        15.0,
                        (0.0, 15.0),
                        (25.0, 40.0),
                        (30.0, 50.0),
                        (0.0, 0.0),
                        (0.0, 15.0),
                        (15.0, 100.0),
                    ),
                );
                biome_params.insert(
                    format!("Rocky Fields"),
                    (
                        15.0,
                        (0.0, 15.0),
                        (25.0, 40.0),
                        (30.0, 50.0),
                        (0.0, 0.0),
                        (0.0, 15.0),
                        (0.0, 15.0),
                    ),
                );
                biome_params.insert(
                    format!("Warped Rocky Fields"),
                    (
                        15.0,
                        (0.0, 15.0),
                        (25.0, 40.0),
                        (30.0, 50.0),
                        (0.0, 0.0),
                        (15.0, 100.0),
                        (15.0, 100.0),
                    ),
                );
                biome_params.insert(
                    format!("Chaotic Rocky Fields"),
                    (
                        15.0,
                        (0.0, 15.0),
                        (25.0, 40.0),
                        (30.0, 50.0),
                        (0.0, 0.0),
                        (0.0, 15.0),
                        (15.0, 100.0),
                    ),
                );
                Self {
                    biome_params: biome_params,
                }
            }
        }
    }
}
