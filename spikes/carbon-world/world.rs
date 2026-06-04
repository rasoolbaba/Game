// world.rs — NATIVE build of the Carbon World generator (throwaway spike).
// Thin wrapper over the SHARED core (carbon_core.rs); prints an ASCII biome map + JSON.
// Same core is compiled to WASM by world_wasm.rs -> identical worlds.
// Compile: rustc -O world.rs -o world      Run: ./world <seed> [map|nomap]

include!("carbon_core.rs");

const CH: [u8; NBIOME] = [b'~', b'-', b'.', b'd', b's', b',', b';', b'f', b'F', b't', b'u', b'^', b'*'];
const NAME: [&str; NBIOME] = ["deepocean","ocean","beach","desert","savanna","grass","shrub","tempforest","rainforest","taiga","tundra","rock","snow"];

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let seed: u64 = args.get(1).and_then(|s| s.parse().ok()).unwrap_or(16);
    let show = args.get(2).map(|s| s != "nomap").unwrap_or(true);

    let mut w = Box::new(World::ZERO);
    w.generate(seed);

    if show {
        for y in 0..H {
            let mut line = String::with_capacity(W as usize);
            for x in 0..W { line.push(CH[w.biome[(y * W + x) as usize] as usize] as char); }
            println!("{}", line);
        }
        let mut leg = String::from("legend:");
        for b in 0..NBIOME { leg.push_str(&format!(" {}={}", CH[b] as char, NAME[b])); }
        println!("{}", leg);
    }

    let total: u32 = w.hist.iter().sum();
    let ocean = w.hist[0] + w.hist[1];
    print!("{{\"seed\":{},\"cells\":{},\"land_pct\":{},\"fingerprint\":\"{}\",\"hist\":{{", seed, total, (total - ocean) * 100 / total, w.fingerprint());
    for b in 0..NBIOME { print!("{}\"{}\":{}", if b > 0 { "," } else { "" }, NAME[b], w.hist[b]); }
    println!("}}}}");
}
