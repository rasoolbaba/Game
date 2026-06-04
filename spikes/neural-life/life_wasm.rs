// life_wasm.rs — WASM build of the neural-life prototype. no_std, no heap, no imports.
// Compiles the SHARED core (core_sim.rs) to wasm32, exposed via a tiny C ABI for the
// browser viewer + the Node determinism check. Because it includes the *same* core_sim.rs
// as the native build, identical seed -> identical fingerprint across native and wasm.
// Build: rustc --target wasm32-unknown-unknown -O life_wasm.rs -o web/life.wasm
#![no_std]
#![crate_type = "cdylib"]
#![allow(static_mut_refs)]

#[panic_handler] fn ph(_: &core::panic::PanicInfo) -> ! { loop {} }

include!("core_sim.rs");

static mut WORLD: World = World::ZERO;

#[no_mangle] pub extern "C" fn init(seed_lo: u32, seed_hi: u32) { unsafe { WORLD.init(((seed_hi as u64) << 32) | (seed_lo as u64)); } }
#[no_mangle] pub extern "C" fn step() { unsafe { WORLD.step(); } }
#[no_mangle] pub extern "C" fn pop() -> u32 { unsafe { WORLD.count as u32 } }
#[no_mangle] pub extern "C" fn cx(i: u32) -> i32 { unsafe { WORLD.cre[i as usize].x } }
#[no_mangle] pub extern "C" fn cy(i: u32) -> i32 { unsafe { WORLD.cre[i as usize].y } }
#[no_mangle] pub extern "C" fn cenergy(i: u32) -> i32 { unsafe { WORLD.cre[i as usize].energy as i32 } } // Q16.16; JS / 65536
#[no_mangle] pub extern "C" fn food_at(x: i32, y: i32) -> u32 { unsafe { if WORLD.food[idx(x, y)] { 1 } else { 0 } } }
#[no_mangle] pub extern "C" fn width() -> i32 { W }
#[no_mangle] pub extern "C" fn height() -> i32 { H }
#[no_mangle] pub extern "C" fn tick() -> u32 { unsafe { WORLD.tick as u32 } }
#[no_mangle] pub extern "C" fn births() -> u32 { unsafe { WORLD.births as u32 } }
#[no_mangle] pub extern "C" fn deaths() -> u32 { unsafe { WORLD.deaths as u32 } }
#[no_mangle] pub extern "C" fn eaten() -> u32 { unsafe { WORLD.eaten as u32 } }
#[no_mangle] pub extern "C" fn fp_lo() -> u32 { unsafe { WORLD.fingerprint() as u32 } }
#[no_mangle] pub extern "C" fn fp_hi() -> u32 { unsafe { (WORLD.fingerprint() >> 32) as u32 } }
