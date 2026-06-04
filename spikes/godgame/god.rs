// god.rs — NATIVE build of the god-game (throwaway spike). Runs a FIXED act-script so the run
// is reproducible and can be cross-checked against wasm. Compile: rustc -O god.rs -o god
include!("god_core.rs");

fn main() {
    let a: Vec<String> = std::env::args().collect();
    let ws: u64 = a.get(1).and_then(|s| s.parse().ok()).unwrap_or(7);
    let ss: u64 = a.get(2).and_then(|s| s.parse().ok()).unwrap_or(1);

    let mut g = Box::new(Game::ZERO);
    g.init(ws, ss, IMPRINT, [(90 * SCALE) / 100, (20 * SCALE) / 100, (20 * SCALE) / 100]);

    let bel0 = g.believers();
    for t in 0..500u64 {
        // fixed divine-act script (same as the Node cross-check)
        if t == 50 { g.verse(30, 20); }
        if t == 120 { g.bounty(20, 28); }
        if t == 220 { g.verse(45, 18); }
        if t == 300 { g.warm(10, 10); }
        if t == 380 { g.verse(32, 22); }
        g.step();
        if g.w.count == 0 { break; }
    }
    println!("{{\"world\":{},\"sim\":{},\"pop\":{},\"believers_start\":{},\"believers_end\":{},\"devotion\":{},\"fingerprint\":\"{}\"}}",
        ws, ss, g.w.count, bel0, g.believers(), g.devotion, g.fingerprint());
}
