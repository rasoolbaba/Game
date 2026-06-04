// life.rs — NATIVE build of the neural-life prototype (throwaway spike, not final game code).
// Thin wrapper over the SHARED core (core_sim.rs); adds CLI args + text/JSON output.
// The exact same core_sim.rs is compiled to WASM by life_wasm.rs → identical results.
// Compile: rustc -O life.rs -o life      Run: ./life <ticks> <seed> [snap]

include!("core_sim.rs");

fn print_snapshot(t: u64, w: &World) {
    let mut grid = [b'.'; NCELL];
    for c in 0..NCELL { if w.food[c] { grid[c] = b','; } }
    for i in 0..w.count { if w.cre[i].alive { grid[idx(w.cre[i].x, w.cre[i].y)] = b'@'; } }
    println!("--- tick {} | pop {} ---", t, w.count);
    for y in 0..H { let mut line = String::new(); for x in 0..W { line.push(grid[idx(x, y)] as char); } println!("{}", line); }
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let ticks: u64 = args.get(1).and_then(|s| s.parse().ok()).unwrap_or(400);
    let seed: u64 = args.get(2).and_then(|s| s.parse().ok()).unwrap_or(GOLDEN);
    let snap = args.get(3).map(|s| s == "snap").unwrap_or(false);

    let mut w = Box::new(World::ZERO);
    w.init(seed);
    let mut peak = w.count;
    if snap { print_snapshot(0, &w); }
    for t in 1..=ticks {
        w.step();
        if w.count > peak { peak = w.count; }
        if snap && (t == ticks / 2 || t == ticks) { print_snapshot(t, &w); }
        if w.count == 0 { break; }
    }
    println!("{{\"impl\":\"neural-life-native\",\"ticks\":{},\"seed\":{},\"final_pop\":{},\"peak_pop\":{},\"births\":{},\"deaths\":{},\"eaten\":{},\"fingerprint\":\"{}\"}}",
        ticks, seed, w.count, peak, w.births, w.deaths, w.eaten, w.fingerprint());
}
