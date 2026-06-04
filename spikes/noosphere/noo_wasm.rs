// noo_wasm.rs — WASM build of the Noosphere prototype. no_std, no heap, no imports.
// Includes shared noo_core.rs -> identical to native (native==wasm verified).
// Build: rustc --target wasm32-unknown-unknown -C opt-level=z -C lto=fat -C panic=abort -C strip=symbols noo_wasm.rs -o web/noo.wasm
#![no_std]
#![crate_type = "cdylib"]
#![allow(static_mut_refs)]

#[panic_handler] fn ph(_: &core::panic::PanicInfo) -> ! { loop {} }

include!("noo_core.rs");

static mut WORLD: World = World::ZERO;

#[no_mangle] pub extern "C" fn init(wlo: u32, whi: u32, slo: u32, shi: u32, imp_on: u32) {
    unsafe { WORLD.init(((whi as u64) << 32) | wlo as u64, ((shi as u64) << 32) | slo as u64, if imp_on != 0 { IMPRINT } else { 0 }); }
}
#[no_mangle] pub extern "C" fn step() { unsafe { WORLD.step(); } }
#[no_mangle] pub extern "C" fn pop() -> u32 { unsafe { WORLD.count as u32 } }
#[no_mangle] pub extern "C" fn cx(i: u32) -> i32 { unsafe { WORLD.cre[i as usize].x } }
#[no_mangle] pub extern "C" fn cy(i: u32) -> i32 { unsafe { WORLD.cre[i as usize].y } }
#[no_mangle] pub extern "C" fn cenergy(i: u32) -> i32 { unsafe { WORLD.cre[i as usize].energy as i32 } }
#[no_mangle] pub extern "C" fn belief(i: u32, k: u32) -> i32 { unsafe { WORLD.cre[i as usize].belief[k as usize] as i32 } }
#[no_mangle] pub extern "C" fn faith(i: u32) -> u32 { unsafe { WORLD.faith(i as usize) as u32 } }
#[no_mangle] pub extern "C" fn biome_at(x: i32, y: i32) -> u32 { unsafe { WORLD.biome[idx(x, y)] as u32 } }
#[no_mangle] pub extern "C" fn food_at(x: i32, y: i32) -> i32 { unsafe { WORLD.food[idx(x, y)] as i32 } }
#[no_mangle] pub extern "C" fn width() -> i32 { GW }
#[no_mangle] pub extern "C" fn height() -> i32 { GH }
#[no_mangle] pub extern "C" fn births() -> u32 { unsafe { WORLD.births as u32 } }
#[no_mangle] pub extern "C" fn deaths() -> u32 { unsafe { WORLD.deaths as u32 } }
#[no_mangle] pub extern "C" fn tickn() -> u32 { unsafe { WORLD.tick as u32 } }
#[no_mangle] pub extern "C" fn fp_lo() -> u32 { unsafe { WORLD.fingerprint() as u32 } }
#[no_mangle] pub extern "C" fn fp_hi() -> u32 { unsafe { (WORLD.fingerprint() >> 32) as u32 } }
