// god_core.rs — the GOD-GAME layer on top of the proven Noosphere sim (Stage 1).
// Reuses noo_core.rs verbatim (include!) and wraps it in a Game that adds the indirect-god loop:
//   - patron belief (the god's "faith"); believers = creatures aligned to it
//   - devotion = the god's power, accrued from believers each tick, spent on acts
//   - INDIRECT acts (free will preserved — creatures may drift back via conformity):
//       verse(x,y)  : plant the patron belief in a region (a divine idea spreads)
//       bounty(x,y) : a feast (fill food) — reward faith / draw life
//       warm(x,y)   : a mild miracle (raise temperature) — ease survival
// Deterministic: world+sim seeds + the timestamped act sequence -> identical run (P2P-syncable).
// Not a novel invention (ADR-0003): an honest god-game loop built on known foundations.

include!("../noosphere/noo_core.rs");

pub const VERSE_COST: i64 = 300;
pub const BOUNTY_COST: i64 = 200;
pub const WARM_COST: i64 = 150;
pub const START_DEVOTION: i64 = 250;
const BELIEVER_THRESH: i64 = (70 * SCALE) / 100; // align >= 0.70  (avg per-channel distance <= 0.30)
const VERSE_PULL: i64 = SCALE / 2;               // 0.5 nudge toward patron (not total -> free will)
const ACT_R: i32 = 5;
const WARM_AMT: i64 = (20 * SCALE) / 100;

#[inline] fn align(b: &[i64; B], p: &[i64; B]) -> i64 {
    let mut d = 0i64; let mut k = 0; while k < B { let x = b[k] - p[k]; d += if x < 0 { -x } else { x }; k += 1; }
    SCALE - d / (B as i64)
}

pub struct Game { pub w: World, pub patron: [i64; B], pub devotion: i64 }

impl Game {
    pub const ZERO: Game = Game { w: World::ZERO, patron: [0; B], devotion: 0 }; // all-zero -> .bss (small wasm); patron is set in init()

    pub fn init(&mut self, world_seed: u64, sim_seed: u64, imprint: i64, patron: [i64; B]) {
        self.w.init(world_seed, sim_seed, imprint);
        self.patron = [clampu(patron[0]), clampu(patron[1]), clampu(patron[2])];
        self.devotion = START_DEVOTION;
    }

    pub fn step(&mut self) {
        self.w.step();
        self.devotion += self.believers() as i64;          // power grows with the faithful
        if self.devotion > 1_000_000 { self.devotion = 1_000_000; }
    }

    pub fn believers(&self) -> u32 {
        let mut b = 0u32; let mut i = 0;
        while i < self.w.count { if align(&self.w.cre[i].belief, &self.patron) >= BELIEVER_THRESH { b += 1; } i += 1; }
        b
    }
    pub fn set_patron(&mut self, b0: i64, b1: i64, b2: i64) { self.patron = [clampu(b0), clampu(b1), clampu(b2)]; }

    pub fn verse(&mut self, x: i32, y: i32) -> bool {
        if self.devotion < VERSE_COST { return false; } self.devotion -= VERSE_COST;
        let mut i = 0;
        while i < self.w.count {
            let dx = self.w.cre[i].x - x; let dy = self.w.cre[i].y - y;
            if dx * dx + dy * dy <= ACT_R * ACT_R {
                let mut k = 0; while k < B { self.w.cre[i].belief[k] = clampu(self.w.cre[i].belief[k] + fmul(VERSE_PULL, self.patron[k] - self.w.cre[i].belief[k])); k += 1; }
            }
            i += 1;
        }
        true
    }
    pub fn bounty(&mut self, x: i32, y: i32) -> bool {
        if self.devotion < BOUNTY_COST { return false; } self.devotion -= BOUNTY_COST;
        let mut dy = -ACT_R; while dy <= ACT_R { let ny = y + dy; if ny >= 0 && ny < GH {
            let mut dx = -ACT_R; while dx <= ACT_R { let nx = x + dx; if nx >= 0 && nx < GW { let j = idx(nx, ny); if self.w.biome[j] != 0 { self.w.food[j] = FOOD_MAX; } } dx += 1; } } dy += 1; }
        true
    }
    pub fn warm(&mut self, x: i32, y: i32) -> bool {
        if self.devotion < WARM_COST { return false; } self.devotion -= WARM_COST;
        let mut dy = -ACT_R; while dy <= ACT_R { let ny = y + dy; if ny >= 0 && ny < GH {
            let mut dx = -ACT_R; while dx <= ACT_R { let nx = x + dx; if nx >= 0 && nx < GW { let j = idx(nx, ny); let t = self.w.temp[j] as i64 + WARM_AMT; self.w.temp[j] = (if t > SCALE { SCALE } else { t }) as i32; } dx += 1; } } dy += 1; }
        true
    }

    pub fn fingerprint(&self) -> u64 { self.w.fingerprint() ^ (self.devotion as u64).wrapping_mul(0x9E3779B97F4A7C15) }
}
