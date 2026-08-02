use fxhash::FxHashMap;
use rayon::prelude::*;

type BiomeParams = (
    usize, // assignment prob (if determenistic: usize::MAX;; otherwise for n different biomes options: descision = prob_i / sum_of_nprob) (assumed to sum up to 100 for now)
    (f64, f64), // highest absolute height difference
    (f64, f64), // percent height (water lvl <-> top of world)
    (f64, f64), // percent moisture
    (f64, f64), // percent rainfall
    (f64, f64), // abs temp
    (f64, f64), // percent magic
    (f64, f64), // percent chaos
);

const CARDINAL_DELTAS: [(isize, isize); 4] = [(1, 0), (-1, 0), (0, 1), (0, -1)];

struct CellSample {
    height_diff: f64,
    height_pct: f64,
    moisture_pct: f64,
    rainfall_pct: f64,
    temp: f64,
    magic_pct: f64,
    chaos_pct: f64,
}

pub fn assign_biome(
    width: usize,
    height: usize,
    terrain_map: &[Vec<f64>],
    moisture_map: &[Vec<f64>],
    rainfall_map: &[Vec<f64>],
    temperature_map: &[Vec<f64>],
    magic_map: &[Vec<f64>],
    chaos_map: &[Vec<f64>],
    water_lvl: f64,
    max_moisture: f64,
    biome_set: BiomeParamPresetVals,
) -> Vec<Vec<Biomes>> {
    let presets = BiomeParamPresets::new(&biome_set);
    let biome_params = presets.biome_params();

    let land_max = terrain_map
        .iter()
        .flatten()
        .copied()
        .fold(f64::NEG_INFINITY, f64::max);
    let land_span = (land_max - water_lvl).max(f64::EPSILON);
    let moisture_denom = max_moisture.max(f64::EPSILON);
    let rainfall_scale = land_rainfall_percent_scale(rainfall_map, terrain_map, water_lvl);

    (0..height)
        .into_par_iter()
        .map(|y| {
            (0..width)
                .map(|x| {
                    let sample = sample_cell(
                        x,
                        y,
                        width,
                        height,
                        terrain_map,
                        moisture_map,
                        rainfall_map,
                        temperature_map,
                        magic_map,
                        chaos_map,
                        water_lvl,
                        land_span,
                        moisture_denom,
                        rainfall_scale,
                    );
                    let hash = cell_hash(x, y);
                    pick_biome(biome_params, x, y, &sample, hash)
                })
                .collect()
        })
        .collect()
}

fn in_range(value: f64, (lo, hi): (f64, f64)) -> bool {
    value >= lo && value <= hi
}

fn sample_cell(
    x: usize,
    y: usize,
    width: usize,
    height: usize,
    terrain_map: &[Vec<f64>],
    moisture_map: &[Vec<f64>],
    rainfall_map: &[Vec<f64>],
    temperature_map: &[Vec<f64>],
    magic_map: &[Vec<f64>],
    chaos_map: &[Vec<f64>],
    water_lvl: f64,
    land_span: f64,
    moisture_denom: f64,
    rainfall_scale: f64,
) -> CellSample {
    let terrain_height = terrain_map[y][x];
    let height_pct = ((terrain_height - water_lvl) / land_span * 100.0).round();
    CellSample {
        height_diff: max_adj_height_diff(x, y, width, height, terrain_map).round(),
        height_pct,
        moisture_pct: (moisture_map[y][x] / moisture_denom * 100.0)
            .clamp(0.0, 100.0)
            .round(),
        rainfall_pct: (rainfall_map[y][x] / rainfall_scale * 100.0)
            .clamp(0.0, 100.0)
            .round(),
        temp: temperature_map[y][x].round(),
        magic_pct: magic_map[y][x].clamp(0.0, 100.0).round(),
        chaos_pct: chaos_map[y][x].clamp(0.0, 100.0).round(),
    }
}

fn max_adj_height_diff(
    x: usize,
    y: usize,
    width: usize,
    height: usize,
    terrain_map: &[Vec<f64>],
) -> f64 {
    let center = terrain_map[y][x];
    CARDINAL_DELTAS
        .iter()
        .filter_map(|&(dx, dy)| {
            let nx = x as isize + dx;
            let ny = y as isize + dy;
            if nx < 0 || ny < 0 || nx as usize >= width || ny as usize >= height {
                return None;
            }
            Some((center - terrain_map[ny as usize][nx as usize]).abs())
        })
        .fold(0.0, f64::max)
}

fn land_rainfall_percent_scale(
    rainfall_map: &[Vec<f64>],
    terrain_map: &[Vec<f64>],
    water_lvl: f64,
) -> f64 {
    let mut values = Vec::new();
    for y in 0..rainfall_map.len() {
        for x in 0..rainfall_map[0].len() {
            if terrain_map[y][x] <= water_lvl {
                continue;
            }
            let v = rainfall_map[y][x];
            if v > 0.0 {
                values.push(v);
            }
        }
    }
    if values.is_empty() {
        return 1.0;
    }
    values.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let idx = ((values.len() - 1) as f64 * 0.98).round() as usize;
    values[idx].max(1.0)
}

fn cell_hash(x: usize, y: usize) -> u64 {
    (x as u64).wrapping_mul(73856093) ^ (y as u64).wrapping_mul(19349663)
}

fn biome_matches(params: &BiomeParams, sample: &CellSample) -> bool {
    let (_prob, height_diff, height_pct, moisture_pct, rainfall_pct, temp, magic_pct, chaos_pct) =
        params;

    in_range(sample.height_diff, *height_diff)
        && in_range(sample.height_pct, *height_pct)
        && in_range(sample.moisture_pct, *moisture_pct)
        && in_range(sample.rainfall_pct, *rainfall_pct)
        && in_range(sample.temp, *temp)
        && in_range(sample.magic_pct, *magic_pct)
        && in_range(sample.chaos_pct, *chaos_pct)
}

fn pick_biome(
    biome_params: &FxHashMap<Biomes, BiomeParams>,
    x: usize,
    y: usize,
    sample: &CellSample,
    hash: u64,
) -> Biomes {
    for biome in BIOME_EVAL_ORDER {
        let Some(params) = biome_params.get(&biome) else {
            continue;
        };
        if params.0 == usize::MAX && biome_matches(params, sample) {
            return biome;
        }
    }

    let mut weighted: Vec<(Biomes, usize)> = Vec::new();
    for biome in BIOME_EVAL_ORDER {
        let Some(params) = biome_params.get(&biome) else {
            continue;
        };
        if params.0 != usize::MAX && biome_matches(params, sample) {
            weighted.push((biome, params.0));
        }
    }

    if weighted.is_empty() {
        panic!(
            "no biome matched cell ({x}, {y}): height_diff={}, height_pct={}, moisture_pct={}, rainfall_pct={}, temp={}, magic_pct={}, chaos_pct={}",
            sample.height_diff,
            sample.height_pct,
            sample.moisture_pct,
            sample.rainfall_pct,
            sample.temp,
            sample.magic_pct,
            sample.chaos_pct,
        );
    }

    let total: usize = weighted.iter().map(|(_, weight)| weight).sum();
    let mut roll = (hash as usize) % total.max(1);
    for &(biome, weight) in &weighted {
        if roll < weight {
            return biome;
        }
        roll -= weight;
    }
    weighted.last().unwrap().0
}

pub enum BiomeParamPresetVals {
    Basic,
}

#[allow(non_camel_case_types)]
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum Biomes {
    GRASSLANDS,
    WARPED_GRASSLANDS,
    CHAOTIC_GRASSLANDS,
    DESERT,
    WARPED_DESERT,
    CHAOTIC_DESERT,
    SAVANNAH,
    WARPED_SAVANNAH,
    CHAOTIC_SAVANNAH,
    STEEP_MOUNTAIN,
    WARPED_STEEP_MOUNTAIN,
    CHAOTIC_STEEP_MOUNTAIN,
    MOUNTAIN_PEAK,
    WARPED_MOUNTAIN_PEAK,
    CHAOTIC_MOUNTAIN_PEAK,
    OCEAN,
    WARPED_OCEAN,
    CHAOTIC_OCEAN,
    DEEP_OCEAN,
    WARPED_DEEP_OCEAN,
    CHAOTIC_DEEP_OCEAN,
    SHALLOW_OCEAN,
    WARPED_SHALLOW_OCEAN,
    CHAOTIC_SHALLOW_OCEAN,
    BADLANDS,
    WARPED_BADLANDS,
    CHAOTIC_BADLANDS,
    FORREST,
    WARPED_FORREST,
    CHAOTIC_FORREST,
    SPARCE_FORREST,
    WARPED_SPARCE_FORREST,
    CHAOTIC_SPARCE_FORREST,
    DEEP_FORREST,
    WARPED_DEEP_FORREST,
    CHAOTIC_DEEP_FORREST,
    RAINFORREST,
    WARPED_RAINFORREST,
    CHAOTIC_RAINFORREST,
    SWAMPY_FORREST,
    WARPED_SWAMPY_FORREST,
    CHAOTIC_SWAMPY_FORREST,
    SWAMP,
    WARPED_SWAMP,
    CHAOTIC_SWAMP,
    ICE_TAIGA,
    WARPED_ICE_TAIGA,
    CHAOTIC_ICE_TAIGA,
    SNOWY_TUNDRA,
    WARPED_SNOWY_TUNDRA,
    CHAOTIC_SNOWY_TUNDRA,
    BOREAL_FORREST,
    WARPED_BOREAL_FORREST,
    CHAOTIC_BOREAL_FORREST,
    ROCKY_FIELDS,
    WARPED_ROCKY_FIELDS,
    CHAOTIC_ROCKY_FIELDS,
}

const BIOME_EVAL_ORDER: [Biomes; 57] = [
    Biomes::GRASSLANDS,
    Biomes::WARPED_GRASSLANDS,
    Biomes::CHAOTIC_GRASSLANDS,
    Biomes::DESERT,
    Biomes::WARPED_DESERT,
    Biomes::CHAOTIC_DESERT,
    Biomes::SAVANNAH,
    Biomes::WARPED_SAVANNAH,
    Biomes::CHAOTIC_SAVANNAH,
    Biomes::STEEP_MOUNTAIN,
    Biomes::WARPED_STEEP_MOUNTAIN,
    Biomes::CHAOTIC_STEEP_MOUNTAIN,
    Biomes::MOUNTAIN_PEAK,
    Biomes::WARPED_MOUNTAIN_PEAK,
    Biomes::CHAOTIC_MOUNTAIN_PEAK,
    Biomes::OCEAN,
    Biomes::WARPED_OCEAN,
    Biomes::CHAOTIC_OCEAN,
    Biomes::DEEP_OCEAN,
    Biomes::WARPED_DEEP_OCEAN,
    Biomes::CHAOTIC_DEEP_OCEAN,
    Biomes::SHALLOW_OCEAN,
    Biomes::WARPED_SHALLOW_OCEAN,
    Biomes::CHAOTIC_SHALLOW_OCEAN,
    Biomes::BADLANDS,
    Biomes::WARPED_BADLANDS,
    Biomes::CHAOTIC_BADLANDS,
    Biomes::FORREST,
    Biomes::WARPED_FORREST,
    Biomes::CHAOTIC_FORREST,
    Biomes::SPARCE_FORREST,
    Biomes::WARPED_SPARCE_FORREST,
    Biomes::CHAOTIC_SPARCE_FORREST,
    Biomes::DEEP_FORREST,
    Biomes::WARPED_DEEP_FORREST,
    Biomes::CHAOTIC_DEEP_FORREST,
    Biomes::RAINFORREST,
    Biomes::WARPED_RAINFORREST,
    Biomes::CHAOTIC_RAINFORREST,
    Biomes::SWAMPY_FORREST,
    Biomes::WARPED_SWAMPY_FORREST,
    Biomes::CHAOTIC_SWAMPY_FORREST,
    Biomes::SWAMP,
    Biomes::WARPED_SWAMP,
    Biomes::CHAOTIC_SWAMP,
    Biomes::ICE_TAIGA,
    Biomes::WARPED_ICE_TAIGA,
    Biomes::CHAOTIC_ICE_TAIGA,
    Biomes::SNOWY_TUNDRA,
    Biomes::WARPED_SNOWY_TUNDRA,
    Biomes::CHAOTIC_SNOWY_TUNDRA,
    Biomes::BOREAL_FORREST,
    Biomes::WARPED_BOREAL_FORREST,
    Biomes::CHAOTIC_BOREAL_FORREST,
    Biomes::ROCKY_FIELDS,
    Biomes::WARPED_ROCKY_FIELDS,
    Biomes::CHAOTIC_ROCKY_FIELDS,
];

pub struct BiomeParamPresets {
    biome_params: fxhash::FxHashMap<Biomes, BiomeParams>,
}

impl BiomeParamPresets {
    pub fn biome_params(&self) -> &FxHashMap<Biomes, BiomeParams> {
        &self.biome_params
    }

    pub fn new(preset: &BiomeParamPresetVals) -> Self {
        let mut biome_params: FxHashMap<Biomes, BiomeParams> = FxHashMap::default();
        match preset {
            // (assignment prob, highest absolute height difference, percent height (water lvl <-> top of world), percent moisture, percent rainfall, abs temp, percent magic, percent chaos)
            BiomeParamPresetVals::Basic => {
                biome_params.insert(
                    Biomes::GRASSLANDS,
                    (
                        90,
                        (0.0, 15.0),
                        (0.0, 15.0),
                        (25.0, 40.0),
                        (15.0, 24.0),
                        (10.0, 30.0),
                        (0.0, 15.0),
                        (0.0, 15.0),
                    ),
                );
                biome_params.insert(
                    Biomes::WARPED_GRASSLANDS,
                    (
                        90,
                        (0.0, 15.0),
                        (0.0, 15.0),
                        (25.0, 40.0),
                        (15.0, 24.0),
                        (10.0, 30.0),
                        (16.0, 100.0),
                        (16.0, 100.0),
                    ),
                );
                biome_params.insert(
                    Biomes::CHAOTIC_GRASSLANDS,
                    (
                        90,
                        (0.0, 15.0),
                        (0.0, 15.0),
                        (25.0, 40.0),
                        (15.0, 24.0),
                        (10.0, 30.0),
                        (0.0, 15.0),
                        (16.0, 100.0),
                    ),
                );
                biome_params.insert(
                    Biomes::DESERT,
                    (
                        usize::MAX,
                        (0.0, 15.0),
                        (0.0, 15.0),
                        (0.0, 15.0),
                        (0.0, 15.0),
                        (30.0, 60.0),
                        (0.0, 15.0),
                        (0.0, 15.0),
                    ),
                );
                biome_params.insert(
                    Biomes::WARPED_DESERT,
                    (
                        usize::MAX,
                        (0.0, 15.0),
                        (0.0, 15.0),
                        (0.0, 15.0),
                        (0.0, 15.0),
                        (30.0, 60.0),
                        (16.0, 100.0),
                        (16.0, 100.0),
                    ),
                );
                biome_params.insert(
                    Biomes::CHAOTIC_DESERT,
                    (
                        usize::MAX,
                        (0.0, 15.0),
                        (0.0, 15.0),
                        (0.0, 15.0),
                        (0.0, 15.0),
                        (30.0, 60.0),
                        (0.0, 15.0),
                        (16.0, 100.0),
                    ),
                );
                biome_params.insert(
                    Biomes::SAVANNAH,
                    (
                        usize::MAX,
                        (0.0, 10.0),
                        (5.0, 15.0),
                        (25.0, 40.0),
                        (20.0, 40.0),
                        (31.0, 45.0),
                        (0.0, 15.0),
                        (0.0, 15.0),
                    ),
                );
                biome_params.insert(
                    Biomes::WARPED_SAVANNAH,
                    (
                        usize::MAX,
                        (0.0, 10.0),
                        (5.0, 15.0),
                        (25.0, 40.0),
                        (20.0, 40.0),
                        (31.0, 45.0),
                        (16.0, 100.0),
                        (16.0, 100.0),
                    ),
                );
                biome_params.insert(
                    Biomes::CHAOTIC_SAVANNAH,
                    (
                        usize::MAX,
                        (0.0, 10.0),
                        (5.0, 15.0),
                        (25.0, 40.0),
                        (20.0, 40.0),
                        (31.0, 45.0),
                        (0.0, 15.0),
                        (16.0, 100.0),
                    ),
                );
                biome_params.insert(
                    Biomes::STEEP_MOUNTAIN,
                    (
                        usize::MAX,
                        (60.0, f64::MAX),
                        (30.0, 100.0),
                        (0.0, 100.0),
                        (0.0, 100.0),
                        (0.0, 50.0),
                        (0.0, 15.0),
                        (0.0, 15.0),
                    ),
                );
                biome_params.insert(
                    Biomes::WARPED_STEEP_MOUNTAIN,
                    (
                        usize::MAX,
                        (60.0, f64::MAX),
                        (30.0, 100.0),
                        (0.0, 100.0),
                        (0.0, 100.0),
                        (0.0, 50.0),
                        (16.0, 100.0),
                        (16.0, 100.0),
                    ),
                );
                biome_params.insert(
                    Biomes::CHAOTIC_STEEP_MOUNTAIN,
                    (
                        usize::MAX,
                        (60.0, f64::MAX),
                        (30.0, 100.0),
                        (0.0, 100.0),
                        (0.0, 100.0),
                        (0.0, 50.0),
                        (0.0, 15.0),
                        (16.0, 100.0),
                    ),
                );
                biome_params.insert(
                    Biomes::MOUNTAIN_PEAK,
                    (
                        usize::MAX,
                        (0.0, 25.0),
                        (85.0, 100.0),
                        (0.0, 100.0),
                        (0.0, 100.0),
                        (0.0, 0.0),
                        (0.0, 15.0),
                        (0.0, 15.0),
                    ),
                );
                biome_params.insert(
                    Biomes::WARPED_MOUNTAIN_PEAK,
                    (
                        usize::MAX,
                        (0.0, 25.0),
                        (85.0, 100.0),
                        (0.0, 100.0),
                        (0.0, 100.0),
                        (0.0, 0.0),
                        (16.0, 100.0),
                        (16.0, 100.0),
                    ),
                );
                biome_params.insert(
                    Biomes::CHAOTIC_MOUNTAIN_PEAK,
                    (
                        usize::MAX,
                        (0.0, 25.0),
                        (85.0, 100.0),
                        (0.0, 100.0),
                        (0.0, 100.0),
                        (0.0, 0.0),
                        (0.0, 15.0),
                        (16.0, 100.0),
                    ),
                );
                biome_params.insert(
                    Biomes::OCEAN,
                    (
                        usize::MAX,
                        (0.0, 0.0),
                        (-25.0, -5.0),
                        (0.0, 100.0),
                        (0.0, 100.0),
                        (0.0, 100.0),
                        (0.0, 15.0),
                        (0.0, 15.0),
                    ),
                );
                biome_params.insert(
                    Biomes::WARPED_OCEAN,
                    (
                        usize::MAX,
                        (0.0, 0.0),
                        (-25.0, -5.0),
                        (0.0, 100.0),
                        (0.0, 100.0),
                        (0.0, 100.0),
                        (16.0, 100.0),
                        (16.0, 100.0),
                    ),
                );
                biome_params.insert(
                    Biomes::CHAOTIC_OCEAN,
                    (
                        usize::MAX,
                        (0.0, 0.0),
                        (-25.0, -5.0),
                        (0.0, 100.0),
                        (0.0, 100.0),
                        (0.0, 100.0),
                        (0.0, 15.0),
                        (16.0, 100.0),
                    ),
                );
                biome_params.insert(
                    Biomes::DEEP_OCEAN,
                    (
                        usize::MAX,
                        (0.0, 0.0),
                        (-100.0, -26.0),
                        (0.0, 100.0),
                        (0.0, 100.0),
                        (0.0, 100.0),
                        (0.0, 15.0),
                        (0.0, 15.0),
                    ),
                );
                biome_params.insert(
                    Biomes::WARPED_DEEP_OCEAN,
                    (
                        usize::MAX,
                        (0.0, 0.0),
                        (-100.0, -26.0),
                        (0.0, 100.0),
                        (0.0, 100.0),
                        (0.0, 100.0),
                        (16.0, 100.0),
                        (16.0, 100.0),
                    ),
                );
                biome_params.insert(
                    Biomes::CHAOTIC_DEEP_OCEAN,
                    (
                        usize::MAX,
                        (0.0, 0.0),
                        (-100.0, -26.0),
                        (0.0, 100.0),
                        (0.0, 100.0),
                        (0.0, 100.0),
                        (0.0, 15.0),
                        (16.0, 100.0),
                    ),
                );
                biome_params.insert(
                    Biomes::SHALLOW_OCEAN,
                    (
                        usize::MAX,
                        (0.0, 0.0),
                        (-4.0, -1.0),
                        (0.0, 100.0),
                        (0.0, 100.0),
                        (0.0, 100.0),
                        (0.0, 15.0),
                        (0.0, 15.0),
                    ),
                );
                biome_params.insert(
                    Biomes::WARPED_SHALLOW_OCEAN,
                    (
                        usize::MAX,
                        (0.0, 0.0),
                        (-4.0, -1.0),
                        (0.0, 100.0),
                        (0.0, 100.0),
                        (0.0, 100.0),
                        (16.0, 100.0),
                        (16.0, 100.0),
                    ),
                );
                biome_params.insert(
                    Biomes::CHAOTIC_SHALLOW_OCEAN,
                    (
                        usize::MAX,
                        (0.0, 0.0),
                        (-4.0, -1.0),
                        (0.0, 100.0),
                        (0.0, 100.0),
                        (0.0, 100.0),
                        (0.0, 15.0),
                        (16.0, 100.0),
                    ),
                );
                biome_params.insert(
                    Biomes::BADLANDS,
                    (
                        usize::MAX,
                        (16.0, 40.0),
                        (0.0, 50.0),
                        (5.0, 19.0),
                        (5.0, 15.0),
                        (30.0, 45.0),
                        (0.0, 15.0),
                        (0.0, 15.0),
                    ),
                );

                biome_params.insert(
                    Biomes::WARPED_BADLANDS,
                    (
                        usize::MAX,
                        (16.0, 40.0),
                        (0.0, 50.0),
                        (5.0, 19.0),
                        (5.0, 15.0),
                        (30.0, 45.0),
                        (16.0, 100.0),
                        (16.0, 100.0),
                    ),
                );
                biome_params.insert(
                    Biomes::CHAOTIC_BADLANDS,
                    (
                        usize::MAX,
                        (16.0, 40.0),
                        (0.0, 50.0),
                        (5.0, 19.0),
                        (5.0, 15.0),
                        (30.0, 45.0),
                        (0.0, 15.0),
                        (16.0, 100.0),
                    ),
                );
                biome_params.insert(
                    Biomes::FORREST,
                    (
                        usize::MAX,
                        (0.0, 40.0),
                        (0.0, 40.0),
                        (26.0, 35.0),
                        (25.0, 45.0),
                        (15.0, 30.0),
                        (0.0, 15.0),
                        (0.0, 15.0),
                    ),
                );
                biome_params.insert(
                    Biomes::WARPED_FORREST,
                    (
                        usize::MAX,
                        (0.0, 40.0),
                        (0.0, 40.0),
                        (26.0, 35.0),
                        (25.0, 45.0),
                        (15.0, 30.0),
                        (16.0, 100.0),
                        (16.0, 100.0),
                    ),
                );
                biome_params.insert(
                    Biomes::CHAOTIC_FORREST,
                    (
                        usize::MAX,
                        (0.0, 40.0),
                        (0.0, 40.0),
                        (26.0, 35.0),
                        (25.0, 45.0),
                        (15.0, 30.0),
                        (0.0, 15.0),
                        (16.0, 100.0),
                    ),
                );
                biome_params.insert(
                    Biomes::SPARCE_FORREST,
                    (
                        10,
                        (0.0, 40.0),
                        (0.0, 40.0),
                        (20.0, 25.0),
                        (15.0, 24.0),
                        (15.0, 30.0),
                        (0.0, 15.0),
                        (0.0, 15.0),
                    ),
                );
                biome_params.insert(
                    Biomes::WARPED_SPARCE_FORREST,
                    (
                        10,
                        (0.0, 40.0),
                        (0.0, 40.0),
                        (20.0, 25.0),
                        (15.0, 24.0),
                        (15.0, 30.0),
                        (16.0, 100.0),
                        (16.0, 100.0),
                    ),
                );
                biome_params.insert(
                    Biomes::CHAOTIC_SPARCE_FORREST,
                    (
                        10,
                        (0.0, 40.0),
                        (0.0, 40.0),
                        (20.0, 25.0),
                        (15.0, 24.0),
                        (15.0, 30.0),
                        (0.0, 15.0),
                        (16.0, 100.0),
                    ),
                );
                biome_params.insert(
                    Biomes::DEEP_FORREST,
                    (
                        usize::MAX,
                        (0.0, 40.0),
                        (0.0, 40.0),
                        (36.0, 40.0),
                        (25.0, 45.0),
                        (15.0, 30.0),
                        (0.0, 15.0),
                        (0.0, 15.0),
                    ),
                );
                biome_params.insert(
                    Biomes::WARPED_DEEP_FORREST,
                    (
                        usize::MAX,
                        (0.0, 40.0),
                        (0.0, 40.0),
                        (36.0, 40.0),
                        (25.0, 45.0),
                        (15.0, 30.0),
                        (16.0, 100.0),
                        (16.0, 100.0),
                    ),
                );
                biome_params.insert(
                    Biomes::CHAOTIC_DEEP_FORREST,
                    (
                        usize::MAX,
                        (0.0, 40.0),
                        (0.0, 40.0),
                        (36.0, 40.0),
                        (25.0, 45.0),
                        (15.0, 30.0),
                        (0.0, 15.0),
                        (16.0, 100.0),
                    ),
                );
                biome_params.insert(
                    Biomes::RAINFORREST,
                    (
                        usize::MAX,
                        (0.0, 40.0),
                        (0.0, 15.0),
                        (41.0, 75.0),
                        (51.0, 75.0),
                        (31.0, 45.0),
                        (0.0, 15.0),
                        (0.0, 15.0),
                    ),
                );
                biome_params.insert(
                    Biomes::WARPED_RAINFORREST,
                    (
                        usize::MAX,
                        (0.0, 40.0),
                        (0.0, 15.0),
                        (41.0, 75.0),
                        (51.0, 75.0),
                        (31.0, 45.0),
                        (16.0, 100.0),
                        (16.0, 100.0),
                    ),
                );
                biome_params.insert(
                    Biomes::CHAOTIC_RAINFORREST,
                    (
                        usize::MAX,
                        (0.0, 40.0),
                        (0.0, 15.0),
                        (41.0, 75.0),
                        (51.0, 75.0),
                        (31.0, 45.0),
                        (0.0, 15.0),
                        (16.0, 100.0),
                    ),
                );
                biome_params.insert(
                    Biomes::SWAMPY_FORREST,
                    (
                        usize::MAX,
                        (0.0, 15.0),
                        (0.0, 15.0),
                        (41.0, 75.0),
                        (51.0, 75.0),
                        (15.0, 30.0),
                        (0.0, 15.0),
                        (0.0, 15.0),
                    ),
                );
                biome_params.insert(
                    Biomes::WARPED_SWAMPY_FORREST,
                    (
                        usize::MAX,
                        (0.0, 15.0),
                        (0.0, 15.0),
                        (41.0, 75.0),
                        (51.0, 75.0),
                        (15.0, 30.0),
                        (16.0, 100.0),
                        (16.0, 100.0),
                    ),
                );
                biome_params.insert(
                    Biomes::CHAOTIC_SWAMPY_FORREST,
                    (
                        usize::MAX,
                        (0.0, 15.0),
                        (0.0, 15.0),
                        (41.0, 75.0),
                        (51.0, 75.0),
                        (15.0, 30.0),
                        (0.0, 15.0),
                        (16.0, 100.0),
                    ),
                );
                biome_params.insert(
                    Biomes::SWAMP,
                    (
                        usize::MAX,
                        (0.0, 15.0),
                        (0.0, 15.0),
                        (41.0, 75.0),
                        (25.0, 50.0),
                        (15.0, 30.0),
                        (0.0, 15.0),
                        (0.0, 15.0),
                    ),
                );

                biome_params.insert(
                    Biomes::WARPED_SWAMP,
                    (
                        usize::MAX,
                        (0.0, 15.0),
                        (0.0, 15.0),
                        (41.0, 75.0),
                        (25.0, 50.0),
                        (15.0, 30.0),
                        (16.0, 100.0),
                        (16.0, 100.0),
                    ),
                );
                biome_params.insert(
                    Biomes::CHAOTIC_SWAMP,
                    (
                        usize::MAX,
                        (0.0, 15.0),
                        (0.0, 15.0),
                        (41.0, 75.0),
                        (25.0, 50.0),
                        (15.0, 30.0),
                        (0.0, 15.0),
                        (16.0, 100.0),
                    ),
                );
                biome_params.insert(
                    Biomes::ICE_TAIGA,
                    (
                        usize::MAX,
                        (0.0, 15.0),
                        (0.0, 40.0),
                        (26.0, 35.0),
                        (10.0, 24.0),
                        (-15.0, 0.0),
                        (0.0, 15.0),
                        (0.0, 15.0),
                    ),
                );
                biome_params.insert(
                    Biomes::WARPED_ICE_TAIGA,
                    (
                        usize::MAX,
                        (0.0, 15.0),
                        (0.0, 40.0),
                        (26.0, 35.0),
                        (10.0, 24.0),
                        (-15.0, 0.0),
                        (16.0, 100.0),
                        (16.0, 100.0),
                    ),
                );
                biome_params.insert(
                    Biomes::CHAOTIC_ICE_TAIGA,
                    (
                        usize::MAX,
                        (0.0, 15.0),
                        (0.0, 40.0),
                        (26.0, 35.0),
                        (10.0, 24.0),
                        (-15.0, 0.0),
                        (0.0, 15.0),
                        (16.0, 100.0),
                    ),
                );
                biome_params.insert(
                    Biomes::SNOWY_TUNDRA,
                    (
                        usize::MAX,
                        (0.0, 15.0),
                        (0.0, 40.0),
                        (20.0, 25.0),
                        (15.0, 24.0),
                        (-15.0, 0.0),
                        (0.0, 15.0),
                        (0.0, 15.0),
                    ),
                );
                biome_params.insert(
                    Biomes::WARPED_SNOWY_TUNDRA,
                    (
                        usize::MAX,
                        (0.0, 15.0),
                        (0.0, 40.0),
                        (20.0, 25.0),
                        (15.0, 24.0),
                        (-15.0, 0.0),
                        (16.0, 100.0),
                        (16.0, 100.0),
                    ),
                );
                biome_params.insert(
                    Biomes::CHAOTIC_SNOWY_TUNDRA,
                    (
                        usize::MAX,
                        (0.0, 15.0),
                        (0.0, 40.0),
                        (20.0, 25.0),
                        (15.0, 24.0),
                        (-15.0, 0.0),
                        (0.0, 15.0),
                        (16.0, 100.0),
                    ),
                );
                biome_params.insert(
                    Biomes::BOREAL_FORREST,
                    (
                        usize::MAX,
                        (0.0, 15.0),
                        (0.0, 40.0),
                        (26.0, 35.0),
                        (25.0, 45.0),
                        (-15.0, 0.0),
                        (0.0, 15.0),
                        (0.0, 15.0),
                    ),
                );
                biome_params.insert(
                    Biomes::WARPED_BOREAL_FORREST,
                    (
                        usize::MAX,
                        (0.0, 15.0),
                        (0.0, 40.0),
                        (26.0, 35.0),
                        (25.0, 45.0),
                        (-15.0, 0.0),
                        (16.0, 100.0),
                        (16.0, 100.0),
                    ),
                );
                biome_params.insert(
                    Biomes::CHAOTIC_BOREAL_FORREST,
                    (
                        usize::MAX,
                        (0.0, 15.0),
                        (0.0, 40.0),
                        (26.0, 35.0),
                        (25.0, 45.0),
                        (-15.0, 0.0),
                        (0.0, 15.0),
                        (16.0, 100.0),
                    ),
                );
                biome_params.insert(
                    Biomes::ROCKY_FIELDS,
                    (
                        usize::MAX,
                        (0.0, 25.0),
                        (16.0, 85.0),
                        (5.0, 24.0),
                        (5.0, 14.0),
                        (5.0, 15.0),
                        (0.0, 15.0),
                        (0.0, 15.0),
                    ),
                );
                biome_params.insert(
                    Biomes::WARPED_ROCKY_FIELDS,
                    (
                        usize::MAX,
                        (0.0, 25.0),
                        (16.0, 85.0),
                        (5.0, 24.0),
                        (5.0, 14.0),
                        (5.0, 15.0),
                        (16.0, 100.0),
                        (16.0, 100.0),
                    ),
                );
                biome_params.insert(
                    Biomes::CHAOTIC_ROCKY_FIELDS,
                    (
                        usize::MAX,
                        (0.0, 25.0),
                        (16.0, 85.0),
                        (5.0, 24.0),
                        (5.0, 14.0),
                        (5.0, 15.0),
                        (0.0, 15.0),
                        (16.0, 100.0),
                    ),
                );
                Self {
                    biome_params: biome_params,
                }
            }
        }
    }
}
