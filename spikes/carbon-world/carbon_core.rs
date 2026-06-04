// carbon_core.rs — SHARED deterministic procedural world core (Stage 1 "Carbon World").
// no_std-compatible: only `core`, fixed arrays, NO heap, NO floats, NO I/O.
// Included by BOTH native (world.rs) and wasm (world_wasm.rs) -> identical worlds, native==wasm.
//
// Pipeline (all integer / Q16.16 fixed-point):
//   elevation = fBm value-noise + contrast            -> continents / oceans
//   moisture  = fBm value-noise x latitude-rainfall    -> subtropical desert belts (~30 deg)
//   current   = low-freq fBm proxy                      -> warm/cold coastal temperature shift
//   temperature = latitude band - altitude lapse + current
//   biome = Whittaker-style classification             -> 13 base biomes
//   rivers = D8 flow accumulation (counting-sort by elevation) -> RIVER / LAKE overlay

pub const SCALE: i64 = 65536; // Q16.16
pub const W: i32 = 96;
pub const H: i32 = 56;
pub const NCELL: usize = (W * H) as usize;
pub const OCTAVES: i32 = 5;
pub const BASE_SPACING: i64 = 48;

const DEEP: i64 = 19660;
const SEA: i64 = 27525;
const BEACH_TOP: i64 = 28835;
const MOUNT: i64 = 47185;
const MOUNT_HIGH: i64 = 53739;
const COLD: i64 = 21626;
const HOT: i64 = 43253;
const DRY: i64 = 21626;
const WET: i64 = 39322;
const LAPSE: i64 = (12 * SCALE) / 10;
const GAIN_E: i64 = (14 * SCALE) / 10;
const GAIN_M: i64 = (18 * SCALE) / 10;
const CUR_STRENGTH: i64 = (14 * SCALE) / 100; // ocean-current temperature proxy
const CUR_SALT: u64 = 0x0CEA_4111_C0FF_EE00;
const MOIST_SALT: u64 = 0xA5A5_5A5A_DEAD_BEEF;

// biomes: 0 deepocean,1 ocean,2 beach,3 desert,4 savanna,5 grass,6 shrub,7 tempforest,
// 8 rainforest,9 taiga,10 tundra,11 rock,12 snow,13 river,14 lake
pub const NBIOME: usize = 15;
const RIVER: u8 = 13;
const LAKE: u8 = 14;
const RIVER_THRESH: i32 = 50;
const LAKE_THRESH: i32 = 120;

const NBUCKET: usize = 1026; // elevation buckets for counting sort (elev>>6 in [0,1024])
const BSHIFT: i32 = 6;

#[inline] fn fmul(a: i64, b: i64) -> i64 { a.wrapping_mul(b) >> 16 }

fn hash2(xi: i32, yi: i32, seed: u64) -> u64 {
    let mut h = seed
        ^ (xi as i64 as u64).wrapping_mul(0x9E3779B97F4A7C15)
        ^ (yi as i64 as u64).wrapping_mul(0xC2B2AE3D27D4EB4F);
    h ^= h >> 33; h = h.wrapping_mul(0xFF51AFD7ED558CCD);
    h ^= h >> 33; h = h.wrapping_mul(0xC4CEB9FE1A85EC53);
    h ^= h >> 33;
    h
}
#[inline] fn lattice(xi: i32, yi: i32, seed: u64) -> i64 { (hash2(xi, yi, seed) & 0xFFFF) as i64 }
#[inline] fn smooth(t: i64) -> i64 { fmul(fmul(t, t), 3 * SCALE - 2 * t) }
#[inline] fn lerp(a: i64, b: i64, t: i64) -> i64 { a + fmul(b - a, t) }
#[inline] fn clampu(v: i64) -> i64 { if v < 0 { 0 } else if v > SCALE { SCALE } else { v } }
#[inline] fn contrast(v: i64, gain: i64) -> i64 { clampu(SCALE / 2 + fmul(v - SCALE / 2, gain)) }

fn vnoise(px: i64, py: i64, seed: u64) -> i64 {
    let xi = (px >> 16) as i32; let yi = (py >> 16) as i32;
    let tx = px & 0xFFFF; let ty = py & 0xFFFF;
    let v00 = lattice(xi, yi, seed); let v10 = lattice(xi + 1, yi, seed);
    let v01 = lattice(xi, yi + 1, seed); let v11 = lattice(xi + 1, yi + 1, seed);
    let sx = smooth(tx); let sy = smooth(ty);
    lerp(lerp(v00, v10, sx), lerp(v01, v11, sx), sy)
}
fn fbm(x: i32, y: i32, seed: u64, octaves: i32, base: i64) -> i64 {
    let mut sum = 0i64; let mut norm = 0i64; let mut o = 0;
    while o < octaves {
        let spacing = if (base >> o) < 1 { 1 } else { base >> o };
        let px = ((x as i64) * SCALE) / spacing;
        let py = ((y as i64) * SCALE) / spacing;
        let amp = SCALE >> o;
        let n = vnoise(px, py, seed.wrapping_add((o as u64).wrapping_mul(0x9E3779B1)));
        sum += fmul(n, amp); norm += amp; o += 1;
    }
    (sum * SCALE) / norm
}

// piecewise-linear rainfall vs |latitude| -> wet equator, dry subtropics (~30 deg), wet temperate, dry poles
fn lerp_seg(x: i64, x0: i64, y0: i64, x1: i64, y1: i64) -> i64 { if x1 == x0 { y0 } else { y0 + ((y1 - y0) * (x - x0)) / (x1 - x0) } }
fn rain_factor(y: i32) -> i64 {
    let half = (H as i64) / 2;
    let absl = (((y as i64) - half).abs() * SCALE) / half;
    let a1 = (15 * SCALE) / 100; let a2 = (35 * SCALE) / 100; let a3 = (70 * SCALE) / 100;
    let f0 = (115 * SCALE) / 100; let f1 = SCALE; let f2 = (40 * SCALE) / 100; let f3 = SCALE; let f4 = (55 * SCALE) / 100;
    if absl <= a1 { lerp_seg(absl, 0, f0, a1, f1) }
    else if absl <= a2 { lerp_seg(absl, a1, f1, a2, f2) }
    else if absl <= a3 { lerp_seg(absl, a2, f2, a3, f3) }
    else { lerp_seg(absl, a3, f3, SCALE, f4) }
}

fn classify(e: i64, m: i64, t: i64) -> u8 {
    if e < DEEP { return 0; }
    if e < SEA { return 1; }
    if e < BEACH_TOP { return 2; }
    if e >= MOUNT_HIGH { return if t < HOT { 12 } else { 11 }; } // peaks: snow-capped unless hot
    if e >= MOUNT { return if t < COLD { 12 } else { 11 }; }      // high: snow if cold, else rock
    if t < COLD { return if m < DRY { 10 } else { 9 }; }
    if t > HOT { return if m < DRY { 3 } else if m < WET { 4 } else { 8 }; }
    if m < DRY { 5 } else if m < WET { 6 } else { 7 }
}

pub struct World {
    pub elev: [i32; NCELL],
    pub temp: [i32; NCELL],
    pub moist: [i32; NCELL],
    pub flow: [i32; NCELL],
    pub biome: [u8; NCELL],
    pub hist: [u32; NBIOME],
    pub seed: u64,
}

impl World {
    pub const ZERO: World = World {
        elev: [0; NCELL], temp: [0; NCELL], moist: [0; NCELL], flow: [0; NCELL],
        biome: [0; NCELL], hist: [0; NBIOME], seed: 0,
    };

    pub fn generate(&mut self, seed: u64) {
        *self = World::ZERO;
        self.seed = seed;
        let half = (H as i64) / 2;
        let mut y = 0;
        while y < H {
            let dist = ((y as i64) - half).abs();
            let lat = SCALE - (dist * SCALE) / half;
            let rf = rain_factor(y);
            let mut x = 0;
            while x < W {
                let i = (y * W + x) as usize;
                let e = contrast(fbm(x, y, seed, OCTAVES, BASE_SPACING), GAIN_E);
                let m = clampu(fmul(contrast(fbm(x, y, seed ^ MOIST_SALT, OCTAVES, BASE_SPACING), GAIN_M), rf));
                let cur = fbm(x, y, seed ^ CUR_SALT, 2, BASE_SPACING * 2); // low-freq current proxy
                let above = if e > SEA { e - SEA } else { 0 };
                let t = clampu(lat - fmul(LAPSE, above) + fmul(CUR_STRENGTH, cur - SCALE / 2));
                self.elev[i] = e as i32; self.moist[i] = m as i32; self.temp[i] = t as i32;
                self.biome[i] = classify(e, m, t);
                x += 1;
            }
            y += 1;
        }
        self.carve_rivers();
        let mut i = 0; while i < NCELL { self.hist[self.biome[i] as usize] += 1; i += 1; }
    }

    // D8 flow accumulation: process cells high->low (counting sort by elevation), push flow to
    // the lowest neighbor; land cells with large drainage become rivers, pits become lakes.
    fn carve_rivers(&mut self) {
        let mut cnt = [0u32; NBUCKET];
        let mut i = 0; while i < NCELL { cnt[(self.elev[i] as usize) >> BSHIFT] += 1; i += 1; }
        // descending positions: highest bucket first
        let mut pos = [0u32; NBUCKET];
        let mut acc = 0u32; let mut b = NBUCKET - 1;
        loop { pos[b] = acc; acc += cnt[b]; if b == 0 { break; } b -= 1; }
        let mut order = [0i32; NCELL];
        let mut tmp = pos;
        i = 0; while i < NCELL { let bk = (self.elev[i] as usize) >> BSHIFT; order[tmp[bk] as usize] = i as i32; tmp[bk] += 1; i += 1; }

        let mut f = 0; while f < NCELL { self.flow[f] = 1; f += 1; }
        let nb: [(i32, i32); 4] = [(-1, 0), (1, 0), (0, -1), (0, 1)];
        let mut k = 0;
        while k < NCELL {
            let idx = order[k] as usize;
            let x = (idx as i32) % W; let y = (idx as i32) / W;
            let ce = self.elev[idx];
            let mut best = -1i32; let mut beste = ce;
            let mut d = 0;
            while d < 4 {
                let nx = x + nb[d].0; let ny = y + nb[d].1;
                if nx >= 0 && nx < W && ny >= 0 && ny < H {
                    let ni = (ny * W + nx) as usize;
                    if self.elev[ni] < beste { beste = self.elev[ni]; best = ni as i32; }
                }
                d += 1;
            }
            if best >= 0 { self.flow[best as usize] += self.flow[idx]; }
            k += 1;
        }
        // overlay rivers/lakes on land
        i = 0;
        while i < NCELL {
            if (self.elev[i] as i64) >= SEA {
                let x = (i as i32) % W; let y = (i as i32) / W;
                let ce = self.elev[i];
                let mut mine = ce; let mut d = 0;
                while d < 4 {
                    let nx = x + nb[d].0; let ny = y + nb[d].1;
                    if nx >= 0 && nx < W && ny >= 0 && ny < H {
                        let ne = self.elev[(ny * W + nx) as usize];
                        if ne < mine { mine = ne; }
                    }
                    d += 1;
                }
                let pit = mine >= ce;
                if pit && self.flow[i] >= LAKE_THRESH { self.biome[i] = LAKE; }
                else if self.flow[i] >= RIVER_THRESH { self.biome[i] = RIVER; }
            }
            i += 1;
        }
    }

    pub fn fingerprint(&self) -> u64 {
        let prime: u64 = 1099511628211;
        let mut h: u64 = 14695981039346656037;
        let mut i = 0;
        while i < NCELL {
            h = (h ^ (self.biome[i] as u64)).wrapping_mul(prime);
            h = (h ^ (self.elev[i] as u32 as u64)).wrapping_mul(prime);
            i += 1;
        }
        h
    }
}
