// carbon_core.rs — SHARED deterministic procedural world core (Stage 1 "Carbon World").
// no_std-compatible: only `core`, fixed arrays, NO heap, NO floats, NO I/O.
// Included by BOTH native (world.rs) and wasm (world_wasm.rs) -> identical worlds, native==wasm.
//
// Pipeline (all integer / Q16.16 fixed-point):
//   elevation = fBm value-noise(seed)          moisture = fBm value-noise(seed^salt)
//   temperature = latitude band - altitude lapse
//   biome = Whittaker-style classification(elevation, moisture, temperature)

pub const SCALE: i64 = 65536; // Q16.16
pub const W: i32 = 96;
pub const H: i32 = 56;
pub const NCELL: usize = (W * H) as usize;
pub const OCTAVES: i32 = 5;
pub const BASE_SPACING: i64 = 48; // largest noise features ~ half the map

// biome thresholds (fraction * SCALE)
const DEEP: i64 = 19660;       // 0.30
const SEA: i64 = 27525;        // 0.42
const BEACH_TOP: i64 = 28835;  // 0.44
const MOUNT: i64 = 47185;      // 0.72
const MOUNT_HIGH: i64 = 53739; // 0.82
const COLD: i64 = 21626;       // 0.33
const HOT: i64 = 43253;        // 0.66
const DRY: i64 = 21626;        // 0.33
const WET: i64 = 39322;        // 0.60
const LAPSE: i64 = (12 * SCALE) / 10; // altitude cooling factor
const GAIN_E: i64 = (14 * SCALE) / 10; // elevation contrast expansion (1.4)
const GAIN_M: i64 = (18 * SCALE) / 10; // moisture contrast expansion (1.8) -> dry & wet extremes

// biome codes: 0 deep ocean,1 ocean,2 beach,3 desert,4 savanna,5 grassland,6 shrubland,
// 7 temperate forest,8 rainforest,9 taiga,10 tundra,11 bare rock,12 snow
pub const NBIOME: usize = 13;

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
#[inline] fn lattice(xi: i32, yi: i32, seed: u64) -> i64 { (hash2(xi, yi, seed) & 0xFFFF) as i64 } // [0,SCALE)
#[inline] fn smooth(t: i64) -> i64 { fmul(fmul(t, t), 3 * SCALE - 2 * t) } // 3t^2-2t^3
#[inline] fn lerp(a: i64, b: i64, t: i64) -> i64 { a + fmul(b - a, t) }

// value noise at fixed-point position (px,py >= 0), Q16.16 -> [0,SCALE]
fn vnoise(px: i64, py: i64, seed: u64) -> i64 {
    let xi = (px >> 16) as i32; let yi = (py >> 16) as i32;
    let tx = px & 0xFFFF; let ty = py & 0xFFFF;
    let v00 = lattice(xi, yi, seed); let v10 = lattice(xi + 1, yi, seed);
    let v01 = lattice(xi, yi + 1, seed); let v11 = lattice(xi + 1, yi + 1, seed);
    let sx = smooth(tx); let sy = smooth(ty);
    lerp(lerp(v00, v10, sx), lerp(v01, v11, sx), sy)
}

// fractal Brownian motion over integer cell coords -> [0,SCALE]
fn fbm(x: i32, y: i32, seed: u64) -> i64 {
    let mut sum = 0i64; let mut norm = 0i64;
    let mut o = 0;
    while o < OCTAVES {
        let spacing = if (BASE_SPACING >> o) < 1 { 1 } else { BASE_SPACING >> o };
        let px = ((x as i64) * SCALE) / spacing;
        let py = ((y as i64) * SCALE) / spacing;
        let amp = SCALE >> o;
        let n = vnoise(px, py, seed.wrapping_add((o as u64).wrapping_mul(0x9E3779B1)));
        sum += fmul(n, amp);
        norm += amp;
        o += 1;
    }
    (sum * SCALE) / norm
}

pub struct World {
    pub elev: [i32; NCELL],
    pub temp: [i32; NCELL],
    pub moist: [i32; NCELL],
    pub biome: [u8; NCELL],
    pub hist: [u32; NBIOME],
    pub seed: u64,
}

#[inline] fn clampu(v: i64) -> i64 { if v < 0 { 0 } else if v > SCALE { SCALE } else { v } }
#[inline] fn contrast(v: i64, gain: i64) -> i64 { clampu(SCALE / 2 + fmul(v - SCALE / 2, gain)) }

fn classify(e: i64, m: i64, t: i64) -> u8 {
    if e < DEEP { return 0; }
    if e < SEA { return 1; }
    if e < BEACH_TOP { return 2; }
    if e >= MOUNT_HIGH { return if t < COLD { 12 } else { 11 }; }
    if e >= MOUNT { return if t < COLD { 12 } else { 11 }; }
    if t < COLD { return if m < DRY { 10 } else { 9 }; }
    if t > HOT { return if m < DRY { 3 } else if m < WET { 4 } else { 8 }; }
    if m < DRY { 5 } else if m < WET { 6 } else { 7 }
}

impl World {
    pub const ZERO: World = World {
        elev: [0; NCELL], temp: [0; NCELL], moist: [0; NCELL],
        biome: [0; NCELL], hist: [0; NBIOME], seed: 0,
    };

    pub fn generate(&mut self, seed: u64) {
        *self = World::ZERO;
        self.seed = seed;
        let half = (H as i64) / 2;
        let mut y = 0;
        while y < H {
            // latitude band: 1.0 at equator (mid), 0 at poles
            let dist = ((y as i64) - half).abs();
            let lat = SCALE - (dist * SCALE) / half;
            let mut x = 0;
            while x < W {
                let i = (y * W + x) as usize;
                let e = contrast(fbm(x, y, seed), GAIN_E);
                let m = contrast(fbm(x, y, seed ^ 0xA5A5_5A5A_DEAD_BEEF), GAIN_M);
                // temperature: latitude minus altitude lapse above sea level
                let above = if e > SEA { e - SEA } else { 0 };
                let t = clampu(lat - fmul(LAPSE, above));
                let b = classify(e, m, t);
                self.elev[i] = e as i32;
                self.moist[i] = m as i32;
                self.temp[i] = t as i32;
                self.biome[i] = b;
                self.hist[b as usize] += 1;
                x += 1;
            }
            y += 1;
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
