// sim_wasm.rs — Rust compiled to wasm32-unknown-unknown (no_std-friendly: only stack arrays,
// no time/args/print). Exposes `run(T) -> acc`. Timing is done by the JS loader.
// Compile: rustc -O --target wasm32-unknown-unknown --crate-type cdylib -C panic=abort
include!("../gen/lut.rs"); // pub const LUT: [i64; 513]

const FBITS: u32 = 16;
const SCALE: i64 = 65536;
const XMIN_FIXED: i64 = -524288;
const XMAX_FIXED: i64 = 524288;
const STEP: i64 = 2048;

#[inline]
fn fmul(a: i64, b: i64) -> i64 { (a.wrapping_mul(b)) >> FBITS }

#[inline]
fn ftanh(x: i64) -> i64 {
    if x <= XMIN_FIXED { return LUT[0]; }
    if x >= XMAX_FIXED { return LUT[LUT.len() - 1]; }
    let pos = x - XMIN_FIXED;
    let idx = (pos / STEP) as usize;
    let frac = pos - (idx as i64) * STEP;
    LUT[idx] + ((LUT[idx + 1] - LUT[idx]) * frac) / STEP
}

struct R { st: u64 }
impl R {
    #[inline]
    fn next(&mut self) -> u64 {
        let mut x = self.st;
        x ^= x << 13; x ^= x >> 7; x ^= x << 17;
        self.st = x; x
    }
}
#[inline]
fn randw(r: &mut R) -> i64 { ((r.next() % 131072) as i64) - 65536 }

#[no_mangle]
pub extern "C" fn run(t_total: u64) -> u64 {
    const IN: usize = 8;
    const HID: usize = 16;
    const OUT: usize = 8;
    let mut r = R { st: 0x9E3779B97F4A7C15u64 };
    let mut w1 = [[0i64; IN]; HID];
    let mut b1 = [0i64; HID];
    let mut w2 = [[0i64; HID]; OUT];
    let mut b2 = [0i64; OUT];
    for j in 0..HID { for i in 0..IN { w1[j][i] = randw(&mut r); } b1[j] = randw(&mut r); }
    for k in 0..OUT { for j in 0..HID { w2[k][j] = randw(&mut r); } b2[k] = randw(&mut r); }
    let mut v = [0i64; IN];
    for i in 0..IN { v[i] = ((i as i64) + 1) * SCALE / 10; }
    let mut acc: u64 = 14695981039346656037;
    let prime: u64 = 1099511628211;
    for _ in 0..t_total {
        let mut h = [0i64; HID];
        for j in 0..HID { let mut s = b1[j]; for i in 0..IN { s += fmul(w1[j][i], v[i]); } h[j] = ftanh(s); }
        let mut o = [0i64; OUT];
        for k in 0..OUT { let mut s = b2[k]; for j in 0..HID { s += fmul(w2[k][j], h[j]); } o[k] = ftanh(s); }
        v = o;
        for k in 0..OUT { acc = (acc ^ (o[k] as u64)).wrapping_mul(prime); }
    }
    acc
}
