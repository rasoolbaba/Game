// tests.js — the red team's falsification suite (run over the deterministic wasm core).
// Distinguishes GENUINE path-dependent emergence from trivial "biome-painting".
//   T1 biome->belief predictiveness (PASS acc<=0.60, FAIL>=0.85)
//   T2 path-dependence: fixed terrain, 16 creature-seeds -> distinct dominant faiths (PASS >=3 & entropy>=1.0)
//   T8 ablation beta=0 (no biome imprint): does path-dependent structure SURVIVE? (the lie-killer)
// Usage: node tests.js [native_fp]
const fs = require('fs'); const path = require('path');
const bytes = fs.readFileSync(path.join(__dirname, 'noo.wasm'));
const nativeFp = process.argv[2];

function mkrun(e) {
  return (world, sim, impON, ticks) => {
    e.init(world >>> 0, 0, sim >>> 0, 0, impON ? 1 : 0);
    for (let t = 0; t < ticks; t++) e.step();
    const n = e.pop(); const cr = new Array(n);
    for (let i = 0; i < n; i++) { const x = e.cx(i), y = e.cy(i); cr[i] = { f: e.faith(i), b: e.biome_at(x, y) }; }
    const fp = (BigInt(e.fp_hi() >>> 0) << 32n) | BigInt(e.fp_lo() >>> 0);
    return { n, cr, fp };
  };
}
const domFaith = (cr) => { const h = new Array(8).fill(0); for (const c of cr) h[c.f]++; let bi = 0, bv = -1; for (let f = 0; f < 8; f++) if (h[f] > bv) { bv = h[f]; bi = f; } return { bi, h }; };
const entropy = (c) => { const t = c.reduce((a, b) => a + b, 0); if (!t) return 0; let H = 0; for (const x of c) if (x > 0) { const p = x / t; H -= p * Math.log2(p); } return H; };
const diversity = (h) => { const t = h.reduce((a, b) => a + b, 0); return t ? h.filter(x => x > 0.05 * t).length : 0; };

function T1(run, world, impON, ticks) {
  const r = run(world, 1, impON, ticks);
  const byb = {}; for (const c of r.cr) { (byb[c.b] = byb[c.b] || new Array(8).fill(0))[c.f]++; }
  const maj = {}; for (const b in byb) { let bi = 0, bv = -1; byb[b].forEach((v, f) => { if (v > bv) { bv = v; bi = f; } }); maj[b] = bi; }
  let correct = 0; for (const c of r.cr) if (maj[c.b] === c.f) correct++;
  return { acc: r.cr.length ? correct / r.cr.length : 0, pop: r.cr.length, diversity: diversity(domFaith(r.cr).h) };
}
function T2(run, world, impON, ticks, N) {
  const counts = new Array(8).fill(0); const outs = [];
  for (let s = 1; s <= N; s++) { const d = domFaith(run(world, s, impON, ticks).cr); outs.push(d.bi); counts[d.bi]++; }
  return { distinct: new Set(outs).size, H: entropy(counts), counts };
}

(async () => {
  const { instance } = await WebAssembly.instantiate(bytes, {});
  const run = mkrun(instance.exports);
  const T = 500, WORLD = 7, N = 16;

  const a = run(WORLD, 1, true, T), b = run(WORLD, 1, true, T);
  console.log('determinism (wasm x2):', a.fp.toString() === b.fp.toString() ? 'PASS ✅' : 'FAIL ❌', a.fp.toString());
  if (nativeFp) console.log('native == wasm:', a.fp.toString() === nativeFp ? 'PASS ✅' : `FAIL ❌ (native ${nativeFp})`);

  const t1on = T1(run, WORLD, true, T), t1off = T1(run, WORLD, false, T);
  console.log(`\nT1 biome->belief predictiveness:`);
  console.log(`   imprintON  acc=${t1on.acc.toFixed(3)}  pop=${t1on.pop}  coexisting_faiths=${t1on.diversity}   (PASS<=0.60, FAIL>=0.85)`);
  console.log(`   imprintOFF acc=${t1off.acc.toFixed(3)}  pop=${t1off.pop}  coexisting_faiths=${t1off.diversity}`);

  const t2on = T2(run, WORLD, true, T, N), t2off = T2(run, WORLD, false, T, N);
  console.log(`\nT2 path-dependence (fixed terrain, ${N} creature-seeds):`);
  console.log(`   imprintON  distinct=${t2on.distinct}  entropy=${t2on.H.toFixed(2)} bits  counts=${JSON.stringify(t2on.counts)}   (PASS: >=3 & >=1.0)`);
  console.log(`T8 ablation beta=0 (no biome imprint):`);
  console.log(`   imprintOFF distinct=${t2off.distinct}  entropy=${t2off.H.toFixed(2)} bits  counts=${JSON.stringify(t2off.counts)}`);

  const T1pass = t1on.acc <= 0.60, T2pass = t2on.distinct >= 3 && t2on.H >= 1.0;
  const T8pass = t2off.distinct >= 3 && t2off.H >= 1.0 && t1off.acc <= 0.60;
  console.log(`\nVERDICT: T1=${T1pass ? 'PASS' : 'FAIL'}  T2=${T2pass ? 'PASS' : 'FAIL'}  T8=${T8pass ? 'PASS' : 'FAIL'}`);
  console.log(T1pass && T2pass && T8pass
    ? '=> Path-dependent emergence on OUR substrate: SUPPORTED. (NB: the phenomenon is known prior art — Axelrod/opinion dynamics. Claim only the integration, ASSUMED.)'
    : '=> Emergence NOT fully supported by T1/T2/T8 -> stays ASSUMED/UNKNOWN; report honestly, tune or do not claim.');
})();
