//! Verify biome param presets for:
//! - invalid intervals (lo > hi)
//! - per-axis projected coverage gaps against expected domains
//! - joint coverage holes (recursive subdivision over climate dims)
//! - unwanted hyperrectangle overlaps (deterministic ∩ deterministic)
//!
//! Run: `cargo run --bin verify_biome_params --release`

use fallen_leaves_world::assign_biome::{BiomeParamPresetVals, BiomeParamPresets, Biomes};
use std::cmp::Ordering;
use std::collections::BTreeSet;
use std::fmt::Write as _;

type Interval = (f64, f64);
type BiomeParams = (
    usize,
    Interval, // height_diff
    Interval, // height_pct
    Interval, // moisture
    Interval, // rainfall
    Interval, // temp
    Interval, // magic
    Interval, // chaos
    Interval, // water_size
    Interval, // ocean_dist
);

const AXIS_NAMES: [&str; 9] = [
    "height_diff",
    "height_pct",
    "moisture_pct",
    "rainfall_pct",
    "temp",
    "magic_pct",
    "chaos_pct",
    "water_body_size",
    "ocean_dist",
];

/// Expected domains for projected saturation checks.
/// Tune these if world-gen rounding / clamping changes.
const EXPECTED_DOMAINS: [Interval; 9] = [
    (0.0, 100.0),    // height_diff
    (-100.0, 100.0), // height_pct
    (0.0, 100.0),    // moisture
    (0.0, 100.0),    // rainfall
    (-20.0, 60.0),   // temp
    (0.0, 100.0),    // magic
    (0.0, 100.0),    // chaos
    (0.0, f64::MAX), // water_body_size
    (0.0, f64::MAX), // ocean_dist
];

/// Climate axes used for joint land coverage (indices into the 9 param intervals).
const CLIMATE_AXES: [usize; 5] = [0, 1, 2, 3, 4]; // hd, hp, mo, rain, temp

/// Land joint-coverage domain (climate only).
const LAND_CLIMATE_DOMAIN: [Interval; 5] = [
    (0.0, 100.0),  // height_diff
    (0.0, 100.0),  // height_pct (land just above water rounds to 0)
    (0.0, 100.0),  // moisture
    (0.0, 100.0),  // rainfall
    (-20.0, 60.0), // temp
];

/// Ocean / inland-water joint-coverage domain (climate only).
/// Water temps can reach TempPreset lower bounds (e.g. -25 on Low).
const OCEAN_CLIMATE_DOMAIN: [Interval; 5] = [
    (0.0, 100.0),
    (-100.0, 0.0),
    (0.0, 100.0),
    (0.0, 100.0),
    (-25.0, 100.0),
];

#[derive(Clone, Copy)]
struct VariantSlice {
    name: &'static str,
    magic: f64,
    chaos: f64,
}

/// Representative (magic, chaos) points for the three variant partitions.
const VARIANT_SLICES: [VariantSlice; 3] = [
    VariantSlice {
        name: "normal",
        magic: 8.0,
        chaos: 8.0,
    },
    VariantSlice {
        name: "warped",
        magic: 50.0,
        chaos: 50.0,
    },
    VariantSlice {
        name: "chaotic",
        magic: 8.0,
        chaos: 50.0,
    },
];

struct BiomeEntry {
    biome: Biomes,
    params: BiomeParams,
}

fn main() {
    let presets = BiomeParamPresets::new(&BiomeParamPresetVals::Basic);
    let mut entries: Vec<BiomeEntry> = presets
        .biome_params()
        .iter()
        .map(|(&biome, &params)| BiomeEntry { biome, params })
        .collect();
    entries.sort_by_key(|e| format!("{:?}", e.biome));

    println!("Biome param verification (Basic preset)");
    println!("biomes loaded: {}\n", entries.len());

    let mut failed = false;

    failed |= report_invalid_intervals(&entries);
    failed |= report_axis_coverage(&entries);
    failed |= report_overlaps(&entries);
    failed |= report_joint_coverage(&entries, "LAND", &LAND_CLIMATE_DOMAIN, |e| is_land_biome(e));
    failed |= report_joint_coverage(&entries, "OCEAN", &OCEAN_CLIMATE_DOMAIN, |e| {
        is_ocean_biome(e)
    });
    failed |= report_joint_coverage(&entries, "INLAND_WATER", &OCEAN_CLIMATE_DOMAIN, |e| {
        is_inland_water_biome(e)
    });

    if failed {
        println!("\nRESULT: FAILED — see gaps / unwanted overlaps above");
        std::process::exit(1);
    } else {
        println!("\nRESULT: OK — no coverage gaps or unwanted deterministic overlaps found");
    }
}

fn intervals_of(p: &BiomeParams) -> [Interval; 9] {
    [p.1, p.2, p.3, p.4, p.5, p.6, p.7, p.8, p.9]
}

fn is_deterministic(p: &BiomeParams) -> bool {
    p.0 == usize::MAX
}

fn base_name(biome: Biomes) -> String {
    let name = format!("{:?}", biome);
    name.strip_prefix("WARPED_")
        .or_else(|| name.strip_prefix("CHAOTIC_"))
        .unwrap_or(&name)
        .to_string()
}

fn is_ocean_biome(e: &BiomeEntry) -> bool {
    let name = format!("{:?}", e.biome);
    name.contains("OCEAN")
}

fn is_inland_water_biome(e: &BiomeEntry) -> bool {
    let name = format!("{:?}", e.biome);
    name.contains("LAKE")
        || name.contains("POND")
        || name.contains("LAGOON")
        || name.contains("SINKHOLE")
}

fn is_land_biome(e: &BiomeEntry) -> bool {
    !is_ocean_biome(e) && !is_inland_water_biome(e)
}

fn interval_valid((lo, hi): Interval) -> bool {
    lo <= hi && lo.is_finite() && (hi.is_finite() || hi == f64::MAX)
}

fn intervals_overlap(a: Interval, b: Interval) -> bool {
    a.0 <= b.1 && b.0 <= a.1
}

fn interval_intersection(a: Interval, b: Interval) -> Option<Interval> {
    let lo = a.0.max(b.0);
    let hi = a.1.min(b.1);
    if lo <= hi { Some((lo, hi)) } else { None }
}

fn hyperrects_overlap(a: &BiomeParams, b: &BiomeParams) -> bool {
    intervals_of(a)
        .into_iter()
        .zip(intervals_of(b))
        .all(|(ia, ib)| intervals_overlap(ia, ib))
}

fn contains_point(iv: Interval, v: f64) -> bool {
    v >= iv.0 && v <= iv.1
}

fn biome_matches_sample(p: &BiomeParams, sample: &[f64; 9]) -> bool {
    intervals_of(p)
        .into_iter()
        .zip(sample.iter().copied())
        .all(|(iv, v)| contains_point(iv, v))
}

fn report_invalid_intervals(entries: &[BiomeEntry]) -> bool {
    println!("=== Invalid intervals ===");
    let mut bad = false;
    for e in entries {
        for (i, iv) in intervals_of(&e.params).into_iter().enumerate() {
            if !interval_valid(iv) {
                bad = true;
                println!(
                    "  {:?}.{} = ({}, {}) is invalid",
                    e.biome, AXIS_NAMES[i], iv.0, iv.1
                );
            }
        }
    }
    if !bad {
        println!("  none");
    }
    println!();
    bad
}

fn merge_intervals(mut ivs: Vec<Interval>) -> Vec<Interval> {
    if ivs.is_empty() {
        return ivs;
    }
    ivs.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(Ordering::Equal));
    let mut out = vec![ivs[0]];
    for iv in ivs.into_iter().skip(1) {
        let last = out.last_mut().unwrap();
        // closed intervals: touching endpoints count as continuous coverage;
        // also merge integer-adjacent ranges like [0,15] ∪ [16,100]
        let touches = iv.0 <= last.1 || (iv.0 - last.1).abs() < f64::EPSILON;
        let int_adjacent =
            last.1.fract() == 0.0 && iv.0.fract() == 0.0 && (iv.0 - last.1 - 1.0).abs() < 1e-9;
        if touches || int_adjacent {
            last.1 = last.1.max(iv.1);
        } else {
            out.push(iv);
        }
    }
    out
}

fn coverage_gaps(domain: Interval, covered: &[Interval], integer_axis: bool) -> Vec<Interval> {
    let mut gaps = Vec::new();
    let mut cursor = domain.0;
    for &(lo, hi) in covered {
        let clo = lo.max(domain.0);
        let chi = hi.min(domain.1);
        if clo > chi {
            continue;
        }
        if clo > cursor {
            // For rounded integer axes, [a,a] then [a+1,b] is continuous.
            let adjacent_ints = integer_axis
                && cursor.fract() == 0.0
                && clo.fract() == 0.0
                && (clo - cursor - 1.0).abs() < 1e-9;
            if !adjacent_ints {
                gaps.push((cursor, clo));
            }
        }
        cursor = cursor.max(chi);
        if cursor >= domain.1 {
            break;
        }
    }
    if cursor < domain.1 {
        let trailing_ok = integer_axis
            && cursor.fract() == 0.0
            && domain.1.fract() == 0.0
            && (domain.1 - cursor - 1.0).abs() < 1e-9;
        if !trailing_ok {
            gaps.push((cursor, domain.1));
        }
    }
    gaps
}

fn report_axis_coverage(entries: &[BiomeEntry]) -> bool {
    println!("=== Per-axis projected coverage ===");
    println!("(union of all biome intervals on each axis vs EXPECTED_DOMAINS)\n");
    let mut bad = false;

    for axis in 0..9 {
        let domain = EXPECTED_DOMAINS[axis];
        let ivs: Vec<Interval> = entries
            .iter()
            .map(|e| intervals_of(&e.params)[axis])
            .filter(|&iv| interval_valid(iv))
            .collect();
        let merged = merge_intervals(ivs);
        // Axes 0..=6 are .round()'d in sample_cell; treat integer adjacency as continuous.
        let integer_axis = axis <= 6;
        let gaps = coverage_gaps(domain, &merged, integer_axis);

        print!(
            "  {:>16} domain [{}, {}]: ",
            AXIS_NAMES[axis],
            fmt_num(domain.0),
            fmt_num(domain.1)
        );
        if gaps.is_empty() {
            println!("SATURATED");
        } else {
            bad = true;
            println!("GAPS:");
            for g in gaps {
                println!("      ({}, {})", fmt_num(g.0), fmt_num(g.1));
            }
        }
    }
    println!();
    bad
}

fn report_overlaps(entries: &[BiomeEntry]) -> bool {
    println!("=== Hyperrectangle overlaps ===");
    println!("unwanted = deterministic ∩ deterministic (eval order silently picks one)");
    println!("info     = involves probabilistic biomes (weighted choice is intentional)\n");

    let mut unwanted = Vec::new();
    let mut info = Vec::new();

    for i in 0..entries.len() {
        for j in (i + 1)..entries.len() {
            let a = &entries[i];
            let b = &entries[j];
            if !hyperrects_overlap(&a.params, &b.params) {
                continue;
            }
            let both_det = is_deterministic(&a.params) && is_deterministic(&b.params);
            let same_family = base_name(a.biome) == base_name(b.biome);
            let note = if same_family {
                "same-family variants"
            } else {
                "cross-biome"
            };
            let line = format!("{:?} ∩ {:?} ({})", a.biome, b.biome, note);
            if both_det {
                unwanted.push(line);
            } else {
                info.push(line);
            }
        }
    }

    println!("  unwanted deterministic overlaps: {}", unwanted.len());
    for line in &unwanted {
        println!("    - {line}");
    }
    if unwanted.is_empty() {
        println!("    none");
    }

    println!("\n  probabilistic overlaps (info): {}", info.len());
    const MAX_INFO: usize = 40;
    for line in info.iter().take(MAX_INFO) {
        println!("    - {line}");
    }
    if info.len() > MAX_INFO {
        println!("    ... and {} more", info.len() - MAX_INFO);
    }
    println!();

    !unwanted.is_empty()
}

fn report_joint_coverage<F>(
    entries: &[BiomeEntry],
    label: &str,
    climate_domain: &[Interval; 5],
    filter: F,
) -> bool
where
    F: Fn(&BiomeEntry) -> bool,
{
    println!("=== Joint climate coverage ({label}) ===");
    println!(
        "recursive subdivision over {:?}; magic/chaos/water/ocean fixed per slice\n",
        CLIMATE_AXES.map(|i| AXIS_NAMES[i])
    );

    let candidates: Vec<&BiomeEntry> = entries.iter().filter(|e| filter(e)).collect();
    if candidates.is_empty() {
        println!("  no biomes in this class\n");
        return true;
    }

    let contexts: &[(&str, f64, f64)] = if label == "OCEAN" {
        &[("ocean", 1_000.0, 0.0)]
    } else if label == "INLAND_WATER" {
        // Representative (water_body_size, ocean_dist) points for each inland basin class.
        &[
            ("pond_or_sinkhole", 8.0, 60.0),
            ("lagoon", 100.0, 25.0),
            ("great_lagoon", 350.0, 25.0),
            ("lake", 100.0, 60.0),
            ("great_lake", 350.0, 60.0),
        ]
    } else {
        // coastal band uses ocean_dist < 5; inland uses >= 5
        &[("coast", 0.0, 2.0), ("inland", 0.0, 10.0)]
    };

    let mut bad = false;
    for &(ctx_name, water, ocean_dist) in contexts {
        for slice in VARIANT_SLICES {
            let fixed = fixed_non_climate(slice, water, ocean_dist);
            let mut gaps = Vec::new();
            find_coverage_gaps(
                climate_domain,
                &candidates,
                &fixed,
                &mut gaps,
                0,
                2_000, // max reported gap cells
            );

            print!("  {ctx_name}/{}: ", slice.name);
            if gaps.is_empty() {
                println!("SATURATED");
            } else {
                bad = true;
                println!("{} uncovered climate cells (showing up to 25):", gaps.len());
                for g in gaps.iter().take(25) {
                    println!("      {g}");
                }
                if gaps.len() > 25 {
                    println!("      ... and {} more", gaps.len() - 25);
                }
            }
        }
    }
    println!();
    bad
}

fn fixed_non_climate(slice: VariantSlice, water: f64, ocean_dist: f64) -> [f64; 9] {
    // Indices: 0 hd, 1 hp, 2 mo, 3 rain, 4 temp, 5 magic, 6 chaos, 7 water, 8 ocean
    let mut sample = [0.0; 9];
    sample[5] = slice.magic;
    sample[6] = slice.chaos;
    sample[7] = water;
    sample[8] = ocean_dist;
    sample
}

fn find_coverage_gaps(
    domain: &[Interval; 5],
    candidates: &[&BiomeEntry],
    fixed: &[f64; 9],
    out: &mut Vec<String>,
    depth: usize,
    max_report: usize,
) {
    if out.len() >= max_report {
        return;
    }

    // Snap climate domain to inclusive integers (matches sample_cell rounding).
    let domain = int_domain(domain);

    // Any biome fully covering this climate box (given fixed non-climate)?
    if candidates
        .iter()
        .any(|e| climate_box_covered_by_biome(&e.params, &domain, fixed))
    {
        return;
    }

    // No biome even intersects this box → whole box is a gap.
    let intersecting: Vec<&&BiomeEntry> = candidates
        .iter()
        .filter(|e| climate_box_intersects_biome(&e.params, &domain, fixed))
        .collect();

    if intersecting.is_empty() {
        out.push(format_climate_box(&domain));
        return;
    }

    // Subdivide along the axis with the most interior integer breakpoints.
    let (axis, cuts) = best_split_axis(&domain, &intersecting);
    if cuts.is_empty() || depth > 40 {
        // Atomic / unsplittable cell: probe every integer point if small, else midpoint.
        let volume = climate_volume(&domain);
        if volume <= 64 {
            for sample in iter_integer_samples(&domain, fixed) {
                let covered = intersecting
                    .iter()
                    .any(|e| biome_matches_sample(&e.params, &sample));
                if !covered {
                    out.push(format!(
                        "{}  (point {:?} uncovered)",
                        format_climate_box(&domain),
                        sample_climate(&sample)
                    ));
                    if out.len() >= max_report {
                        return;
                    }
                }
            }
        } else {
            let mut sample = *fixed;
            for (i, &ax) in CLIMATE_AXES.iter().enumerate() {
                sample[ax] = ((domain[i].0 + domain[i].1) * 0.5).round();
            }
            let covered = intersecting
                .iter()
                .any(|e| biome_matches_sample(&e.params, &sample));
            if !covered {
                out.push(format!(
                    "{}  (mid {:?} uncovered)",
                    format_climate_box(&domain),
                    sample_climate(&sample)
                ));
            }
        }
        return;
    }

    let (lo, hi) = domain[axis];
    // Integer partition: [... cut] then [cut+1 ...]; cuts are biome hi values.
    let mut cut_highs: Vec<i64> = cuts
        .into_iter()
        .map(|c| c.round() as i64)
        .filter(|&c| c >= lo as i64 && c < hi as i64)
        .collect();
    cut_highs.sort_unstable();
    cut_highs.dedup();

    let mut start = lo as i64;
    let end = hi as i64;
    let mut segments: Vec<(i64, i64)> = Vec::new();
    for &cut in &cut_highs {
        if cut >= start {
            segments.push((start, cut));
            start = cut + 1;
        }
    }
    if start <= end {
        segments.push((start, end));
    }

    for (a, b) in segments {
        if out.len() >= max_report {
            return;
        }
        let mut child = domain;
        child[axis] = (a as f64, b as f64);
        find_coverage_gaps(&child, candidates, fixed, out, depth + 1, max_report);
    }
}

fn int_domain(domain: &[Interval; 5]) -> [Interval; 5] {
    let mut out = *domain;
    for iv in &mut out {
        iv.0 = iv.0.ceil();
        iv.1 = iv.1.floor();
    }
    out
}

fn climate_volume(domain: &[Interval; 5]) -> i64 {
    let mut v = 1i64;
    for &(lo, hi) in domain {
        let n = (hi as i64) - (lo as i64) + 1;
        if n <= 0 {
            return 0;
        }
        v = v.saturating_mul(n);
    }
    v
}

fn iter_integer_samples<'a>(
    domain: &'a [Interval; 5],
    fixed: &'a [f64; 9],
) -> impl Iterator<Item = [f64; 9]> + 'a {
    let ranges: Vec<Vec<f64>> = domain
        .iter()
        .map(|&(lo, hi)| {
            let mut v = Vec::new();
            let mut x = lo as i64;
            let end = hi as i64;
            while x <= end {
                v.push(x as f64);
                x += 1;
            }
            v
        })
        .collect();

    // Nested product via index counter
    let lens: Vec<usize> = ranges.iter().map(|r| r.len()).collect();
    let total = lens.iter().fold(1usize, |a, &b| a.saturating_mul(b));
    (0..total).filter_map(move |idx| {
        if lens.iter().any(|&l| l == 0) {
            return None;
        }
        let mut sample = *fixed;
        let mut rem = idx;
        for i in (0..5).rev() {
            let l = lens[i];
            let j = rem % l;
            rem /= l;
            sample[CLIMATE_AXES[i]] = ranges[i][j];
        }
        Some(sample)
    })
}

fn climate_box_covered_by_biome(p: &BiomeParams, domain: &[Interval; 5], fixed: &[f64; 9]) -> bool {
    let ivs = intervals_of(p);
    // Fixed non-climate axes must be inside biome.
    for ax in [5usize, 6, 7, 8] {
        if !contains_point(ivs[ax], fixed[ax]) {
            return false;
        }
    }
    // Biome climate intervals must contain the whole domain box.
    for (i, &ax) in CLIMATE_AXES.iter().enumerate() {
        let (dlo, dhi) = domain[i];
        let (blo, bhi) = ivs[ax];
        if blo > dlo || bhi < dhi {
            return false;
        }
    }
    true
}

fn climate_box_intersects_biome(p: &BiomeParams, domain: &[Interval; 5], fixed: &[f64; 9]) -> bool {
    let ivs = intervals_of(p);
    for ax in [5usize, 6, 7, 8] {
        if !contains_point(ivs[ax], fixed[ax]) {
            return false;
        }
    }
    for (i, &ax) in CLIMATE_AXES.iter().enumerate() {
        if interval_intersection(domain[i], ivs[ax]).is_none() {
            return false;
        }
    }
    true
}

fn best_split_axis(domain: &[Interval; 5], intersecting: &[&&BiomeEntry]) -> (usize, Vec<f64>) {
    let mut best_axis = 0;
    let mut best_cuts = Vec::new();

    for axis in 0..5 {
        let (dlo, dhi) = domain[axis];
        let ax = CLIMATE_AXES[axis];
        let mut cuts = BTreeSet::new();
        for e in intersecting {
            let (_blo, bhi) = intervals_of(&e.params)[ax];
            // Split after biome hi so left child ends at bhi and right starts at bhi+1.
            if bhi >= dlo && bhi < dhi {
                cuts.insert(OrderedF64(bhi.round()));
            }
        }
        if cuts.len() > best_cuts.len() {
            best_axis = axis;
            best_cuts = cuts.into_iter().map(|o| o.0).collect();
        }
    }
    (best_axis, best_cuts)
}

#[derive(Clone, Copy, PartialEq)]
struct OrderedF64(f64);
impl Eq for OrderedF64 {}
impl PartialOrd for OrderedF64 {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
impl Ord for OrderedF64 {
    fn cmp(&self, other: &Self) -> Ordering {
        self.0.partial_cmp(&other.0).unwrap_or(Ordering::Equal)
    }
}

fn format_climate_box(domain: &[Interval; 5]) -> String {
    let mut s = String::new();
    for (i, &ax) in CLIMATE_AXES.iter().enumerate() {
        if i > 0 {
            s.push_str(" | ");
        }
        let _ = write!(
            s,
            "{}:[{}, {}]",
            AXIS_NAMES[ax],
            fmt_num(domain[i].0),
            fmt_num(domain[i].1)
        );
    }
    s
}

fn sample_climate(sample: &[f64; 9]) -> [f64; 5] {
    [sample[0], sample[1], sample[2], sample[3], sample[4]]
}

fn fmt_num(v: f64) -> String {
    if v == f64::MAX {
        "MAX".to_string()
    } else if v == f64::NEG_INFINITY {
        "-INF".to_string()
    } else if v.fract() == 0.0 && v.abs() < 1e15 {
        format!("{v:.0}")
    } else {
        format!("{v:.4}")
    }
}
