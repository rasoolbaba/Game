// core_sim.rs — SHARED deterministic neural-life core.
// no_std-compatible: uses only `core`, fixed-size arrays, NO heap, NO floats, NO I/O.
// Included verbatim by BOTH the native build (life.rs) and the WASM build (life_wasm.rs)
// via include!(), so the two binaries run BIT-IDENTICAL logic. This is what lets us prove
// native == wasm determinism (the cross-runtime risk the red team flagged).

pub const SCALE: i64 = 65536; // Q16.16
pub const W: i32 = 28;
pub const H: i32 = 28;
pub const IN: usize = 4;
pub const HID: usize = 6;
pub const ACT: usize = 5;
pub const SENSE_R: i32 = 7;
pub const MAX_ENERGY: i64 = 10 * SCALE;
pub const START_ENERGY: i64 = 5 * SCALE;
pub const METAB: i64 = SCALE / 10;
pub const FOOD_ENERGY: i64 = 3 * SCALE;
pub const REPRO_THRESH: i64 = 8 * SCALE;
pub const FOOD_PER_TICK: i32 = 6;
pub const CAP: usize = 600;
pub const NCELL: usize = (W * H) as usize; // 784
pub const GOLDEN: u64 = 0x9E3779B97F4A7C15;

#[inline] fn fmul(a: i64, b: i64) -> i64 { a.wrapping_mul(b) >> 16 }
#[inline] fn wrap(v: i32, m: i32) -> i32 { ((v % m) + m) % m }
#[inline] fn idx(x: i32, y: i32) -> usize { (y * W + x) as usize }

#[derive(Clone, Copy)]
pub struct Rng { pub s: u64 }
impl Rng {
    #[inline] fn next(&mut self) -> u64 { let mut x = self.s; x ^= x << 13; x ^= x >> 7; x ^= x << 17; self.s = x; x }
    #[inline] fn rangei(&mut self, n: i64) -> i64 { (self.next() % (n as u64)) as i64 }
    #[inline] fn weight(&mut self) -> i64 { (self.next() % ((2 * SCALE) as u64)) as i64 - SCALE }
}

#[derive(Clone, Copy)]
pub struct Brain { w1: [[i64; IN]; HID], b1: [i64; HID], w2: [[i64; HID]; ACT], b2: [i64; ACT] }
impl Brain {
    const ZERO: Brain = Brain { w1: [[0; IN]; HID], b1: [0; HID], w2: [[0; HID]; ACT], b2: [0; ACT] };
    fn random(seed: u64) -> Brain {
        let mut r = Rng { s: seed | 1 };
        let mut b = Brain::ZERO;
        let mut j = 0; while j < HID { let mut i = 0; while i < IN { b.w1[j][i] = r.weight(); i += 1; } b.b1[j] = r.weight(); j += 1; }
        let mut k = 0; while k < ACT { let mut j2 = 0; while j2 < HID { b.w2[k][j2] = r.weight(); j2 += 1; } b.b2[k] = r.weight(); k += 1; }
        b
    }
    fn mutate(&self, r: &mut Rng) -> Brain {
        let mut c = *self;
        let m = SCALE / 16;
        let mut j = 0; while j < HID { let mut i = 0; while i < IN { c.w1[j][i] += r.rangei(2 * m) - m; i += 1; } c.b1[j] += r.rangei(2 * m) - m; j += 1; }
        let mut k = 0; while k < ACT { let mut j2 = 0; while j2 < HID { c.w2[k][j2] += r.rangei(2 * m) - m; j2 += 1; } c.b2[k] += r.rangei(2 * m) - m; k += 1; }
        c
    }
    fn decide(&self, inp: [i64; IN]) -> usize {
        let mut h = [0i64; HID];
        let mut j = 0; while j < HID { let mut s = self.b1[j]; let mut i = 0; while i < IN { s += fmul(self.w1[j][i], inp[i]); i += 1; } h[j] = if s > 0 { s } else { 0 }; j += 1; }
        let mut best = 0usize; let mut bestv = i64::MIN;
        let mut k = 0; while k < ACT { let mut s = self.b2[k]; let mut j2 = 0; while j2 < HID { s += fmul(self.w2[k][j2], h[j2]); j2 += 1; } if s > bestv { bestv = s; best = k; } k += 1; }
        best
    }
}

#[derive(Clone, Copy)]
pub struct Creature { pub x: i32, pub y: i32, pub energy: i64, brain: Brain, pub genome: u64, pub alive: bool }
impl Creature { const BLANK: Creature = Creature { x: 0, y: 0, energy: 0, brain: Brain::ZERO, genome: 0, alive: false }; }

pub struct World {
    pub food: [bool; NCELL],
    pub cre: [Creature; CAP],
    pub count: usize,
    pub rng: Rng,
    pub genome_ctr: u64,
    pub tick: u64,
    pub births: u64,
    pub deaths: u64,
    pub eaten: u64,
}

impl World {
    pub const ZERO: World = World {
        food: [false; NCELL], cre: [Creature::BLANK; CAP], count: 0,
        rng: Rng { s: 0 }, genome_ctr: 0, tick: 0, births: 0, deaths: 0, eaten: 0,
    };

    pub fn init(&mut self, seed: u64) {
        *self = World::ZERO;
        self.rng = Rng { s: seed | 1 };
        // initial food (same draw order as before refactor)
        let mut i = 0; while i < NCELL / 5 { let c = (self.rng.next() % (NCELL as u64)) as usize; self.food[c] = true; i += 1; }
        // initial creatures
        let mut k = 0; while k < 30 {
            self.genome_ctr = self.genome_ctr.wrapping_add(GOLDEN);
            let g = seed ^ self.genome_ctr;
            let x = (self.rng.next() % (W as u64)) as i32;
            let y = (self.rng.next() % (H as u64)) as i32;
            self.cre[self.count] = Creature { x, y, energy: START_ENERGY, brain: Brain::random(g), genome: g, alive: true };
            self.count += 1; k += 1;
        }
    }

    pub fn step(&mut self) {
        // regrow food
        let mut f = 0; while f < FOOD_PER_TICK { let c = (self.rng.next() % (NCELL as u64)) as usize; self.food[c] = true; f += 1; }
        let n = self.count;       // only pre-existing creatures act this tick
        let mut child = 0usize;   // children appended at indices n..n+child (act next tick)
        let mut i = 0;
        while i < n {
            if !self.cre[i].alive { i += 1; continue; }
            let cx = self.cre[i].x; let cy = self.cre[i].y;
            // perceive: nearest food within radius (Manhattan), direction scaled by SENSE_R
            let mut bestd = i32::MAX; let mut bdx = 0i32; let mut bdy = 0i32;
            let mut dy = -SENSE_R; while dy <= SENSE_R { let mut dx = -SENSE_R; while dx <= SENSE_R {
                let nx = wrap(cx + dx, W); let ny = wrap(cy + dy, H);
                if self.food[idx(nx, ny)] { let d = dx.abs() + dy.abs(); if d < bestd { bestd = d; bdx = dx; bdy = dy; } }
                dx += 1; } dy += 1; }
            let dirx = if bestd == i32::MAX { 0 } else { (bdx as i64 * SCALE) / (SENSE_R as i64) };
            let diry = if bestd == i32::MAX { 0 } else { (bdy as i64 * SCALE) / (SENSE_R as i64) };
            let en = (self.cre[i].energy * SCALE) / MAX_ENERGY;
            let a = self.cre[i].brain.decide([dirx, diry, en, SCALE]);
            let (nx, ny) = match a { 1 => (cx, wrap(cy - 1, H)), 2 => (cx, wrap(cy + 1, H)), 3 => (wrap(cx - 1, W), cy), 4 => (wrap(cx + 1, W), cy), _ => (cx, cy) };
            self.cre[i].x = nx; self.cre[i].y = ny;
            if self.food[idx(nx, ny)] { self.food[idx(nx, ny)] = false; self.cre[i].energy += FOOD_ENERGY; self.eaten += 1; if self.cre[i].energy > MAX_ENERGY { self.cre[i].energy = MAX_ENERGY; } }
            self.cre[i].energy -= METAB;
            if self.cre[i].energy <= 0 { self.cre[i].alive = false; self.deaths += 1; i += 1; continue; }
            if self.cre[i].energy >= REPRO_THRESH && n + child < CAP {
                let half = self.cre[i].energy / 2; self.cre[i].energy = half;
                self.genome_ctr = self.genome_ctr.wrapping_add(GOLDEN);
                let g = self.cre[i].genome ^ self.genome_ctr ^ self.rng.next();
                let mut mr = Rng { s: g | 1 };
                let cb = self.cre[i].brain.mutate(&mut mr);
                self.cre[n + child] = Creature { x: wrap(cx + 1, W), y: cy, energy: half, brain: cb, genome: g, alive: true };
                child += 1; self.births += 1;
            }
            i += 1;
        }
        self.count = n + child;
        // compact: retain alive, stable order (== Vec::retain semantics)
        let mut w = 0; let mut r = 0;
        while r < self.count { if self.cre[r].alive { if w != r { self.cre[w] = self.cre[r]; } w += 1; } r += 1; }
        self.count = w;
        self.tick += 1;
    }

    pub fn fingerprint(&self) -> u64 {
        let prime: u64 = 1099511628211;
        let mut h: u64 = 14695981039346656037;
        let mut i = 0; while i < self.count {
            h = (h ^ (self.cre[i].x as u64)).wrapping_mul(prime);
            h = (h ^ (self.cre[i].y as u64)).wrapping_mul(prime);
            h = (h ^ (self.cre[i].energy as u64)).wrapping_mul(prime);
            h = (h ^ self.cre[i].genome).wrapping_mul(prime);
            i += 1;
        }
        h
    }
}
