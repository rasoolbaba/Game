// noo.rs — NATIVE build of the Noosphere integration prototype (throwaway spike).
// Thin wrapper over shared noo_core.rs (same core compiled to wasm by noo_wasm.rs).
// Compile: rustc -O noo.rs -o noo   Run: ./noo <world_seed> <sim_seed> <imprint 0|1> <ticks> [map]

include!("noo_core.rs");

const FAITHCH: [u8; 8] = [b'.', b'r', b'g', b'y', b'b', b'm', b'c', b'W'];

fn main() {
    let a: Vec<String> = std::env::args().collect();
    let ws: u64 = a.get(1).and_then(|s| s.parse().ok()).unwrap_or(7);
    let ss: u64 = a.get(2).and_then(|s| s.parse().ok()).unwrap_or(1);
    let imp_on: bool = a.get(3).map(|s| s != "0").unwrap_or(true);
    let ticks: u64 = a.get(4).and_then(|s| s.parse().ok()).unwrap_or(600);
    let show = a.get(5).map(|s| s == "map").unwrap_or(false);

    let mut w = Box::new(World::ZERO);
    w.init(ws, ss, if imp_on { IMPRINT } else { 0 });
    let mut peak = w.count;
    for _ in 0..ticks { w.step(); if w.count > peak { peak = w.count; } if w.count == 0 { break; } }

    let mut fh = [0u32; 8];
    let mut i = 0; while i < w.count { fh[w.faith(i) as usize] += 1; i += 1; }

    if show {
        let mut grid = [b' '; NCELL];
        for c in 0..NCELL { grid[c] = if w.biome[c] == 0 { b'~' } else { b'.' }; }
        for c in 0..w.count { grid[idx(w.cre[c].x, w.cre[c].y)] = FAITHCH[w.faith(c) as usize]; }
        for y in 0..GH { let mut l = String::new(); for x in 0..GW { l.push(grid[idx(x, y)] as char); } println!("{}", l); }
    }

    print!("{{\"world\":{},\"sim\":{},\"imprint\":{},\"ticks\":{},\"final_pop\":{},\"peak\":{},\"births\":{},\"deaths\":{},\"faith_hist\":[",
        ws, ss, if imp_on { 1 } else { 0 }, ticks, w.count, peak, w.births, w.deaths);
    for f in 0..8 { print!("{}{}", if f > 0 { "," } else { "" }, fh[f]); }
    println!("],\"fingerprint\":\"{}\"}}", w.fingerprint());
}
