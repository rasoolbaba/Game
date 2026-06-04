// world_wasm.rs — WASM build of the Carbon World. no_std, no heap, no imports.
// Includes the SHARED carbon_core.rs -> identical worlds to native (native==wasm verified).
// Build: rustc --target wasm32-unknown-unknown -C opt-level=z -C lto=fat -C panic=abort -C strip=symbols world_wasm.rs -o web/world.wasm
#![no_std]
#![crate_type = "cdylib"]
#![allow(static_mut_refs)]

#[panic_handler] fn ph(_: &core::panic::PanicInfo) -> ! { loop {} }

include!("carbon_core.rs");

static mut WORLD: World = World::ZERO;

#[no_mangle] pub extern "C" fn generate(seed_lo: u32, seed_hi: u32) { unsafe { WORLD.generate(((seed_hi as u64) << 32) | (seed_lo as u64)); } }
#[no_mangle] pub extern "C" fn width() -> i32 { W }
#[no_mangle] pub extern "C" fn height() -> i32 { H }
#[no_mangle] pub extern "C" fn biome_at(x: i32, y: i32) -> u32 { unsafe { WORLD.biome[(y * W + x) as usize] as u32 } }
#[no_mangle] pub extern "C" fn elev_at(x: i32, y: i32) -> i32 { unsafe { WORLD.elev[(y * W + x) as usize] } }
#[no_mangle] pub extern "C" fn temp_at(x: i32, y: i32) -> i32 { unsafe { WORLD.temp[(y * W + x) as usize] } }
#[no_mangle] pub extern "C" fn moist_at(x: i32, y: i32) -> i32 { unsafe { WORLD.moist[(y * W + x) as usize] } }
#[no_mangle] pub extern "C" fn hist(b: u32) -> u32 { unsafe { if (b as usize) < NBIOME { WORLD.hist[b as usize] } else { 0 } } }
#[no_mangle] pub extern "C" fn nbiome() -> u32 { NBIOME as u32 }
#[no_mangle] pub extern "C" fn fp_lo() -> u32 { unsafe { WORLD.fingerprint() as u32 } }
#[no_mangle] pub extern "C" fn fp_hi() -> u32 { unsafe { (WORLD.fingerprint() >> 32) as u32 } }
