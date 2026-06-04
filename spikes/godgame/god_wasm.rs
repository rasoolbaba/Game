// god_wasm.rs — WASM build of the god-game. no_std, no heap, no imports.
// Build: rustc --target wasm32-unknown-unknown -C opt-level=z -C lto=fat -C panic=abort -C strip=symbols god_wasm.rs -o web/god.wasm
#![no_std]
#![crate_type = "cdylib"]
#![allow(static_mut_refs)]

#[panic_handler] fn ph(_: &core::panic::PanicInfo) -> ! { loop {} }

include!("god_core.rs");

static mut G: Game = Game::ZERO;

#[no_mangle] pub extern "C" fn init(wlo: u32, whi: u32, slo: u32, shi: u32, imp_on: u32, p0: i32, p1: i32, p2: i32) {
    unsafe { G.init(((whi as u64) << 32) | wlo as u64, ((shi as u64) << 32) | slo as u64, if imp_on != 0 { IMPRINT } else { 0 }, [p0 as i64, p1 as i64, p2 as i64]); }
}
#[no_mangle] pub extern "C" fn step() { unsafe { G.step(); } }
#[no_mangle] pub extern "C" fn pop() -> u32 { unsafe { G.w.count as u32 } }
#[no_mangle] pub extern "C" fn cx(i: u32) -> i32 { unsafe { G.w.cre[i as usize].x } }
#[no_mangle] pub extern "C" fn cy(i: u32) -> i32 { unsafe { G.w.cre[i as usize].y } }
#[no_mangle] pub extern "C" fn belief(i: u32, k: u32) -> i32 { unsafe { G.w.cre[i as usize].belief[k as usize] as i32 } }
#[no_mangle] pub extern "C" fn biome_at(x: i32, y: i32) -> u32 { unsafe { G.w.biome[idx(x, y)] as u32 } }
#[no_mangle] pub extern "C" fn food_at(x: i32, y: i32) -> i32 { unsafe { G.w.food[idx(x, y)] as i32 } }
#[no_mangle] pub extern "C" fn width() -> i32 { GW }
#[no_mangle] pub extern "C" fn height() -> i32 { GH }
#[no_mangle] pub extern "C" fn tickn() -> u32 { unsafe { G.w.tick as u32 } }
#[no_mangle] pub extern "C" fn devotion() -> i32 { unsafe { G.devotion as i32 } }
#[no_mangle] pub extern "C" fn believers() -> u32 { unsafe { G.believers() } }
#[no_mangle] pub extern "C" fn patron(k: u32) -> i32 { unsafe { G.patron[k as usize] as i32 } }
#[no_mangle] pub extern "C" fn set_patron(b0: i32, b1: i32, b2: i32) { unsafe { G.set_patron(b0 as i64, b1 as i64, b2 as i64); } }
#[no_mangle] pub extern "C" fn verse(x: i32, y: i32) -> u32 { unsafe { if G.verse(x, y) { 1 } else { 0 } } }
#[no_mangle] pub extern "C" fn bounty(x: i32, y: i32) -> u32 { unsafe { if G.bounty(x, y) { 1 } else { 0 } } }
#[no_mangle] pub extern "C" fn warm(x: i32, y: i32) -> u32 { unsafe { if G.warm(x, y) { 1 } else { 0 } } }
#[no_mangle] pub extern "C" fn fp_lo() -> u32 { unsafe { G.fingerprint() as u32 } }
#[no_mangle] pub extern "C" fn fp_hi() -> u32 { unsafe { (G.fingerprint() >> 32) as u32 } }
