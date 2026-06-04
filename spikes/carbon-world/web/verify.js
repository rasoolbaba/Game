// verify.js — headless cross-runtime determinism check (Node).
// Instantiates world.wasm, generates seed 16, compares WASM fingerprint to the NATIVE one
// passed as argv[2]. Proves: same Rust core, native == wasm.
const fs = require('fs');
const path = require('path');
const bytes = fs.readFileSync(path.join(__dirname, 'world.wasm'));
const expected = process.argv[2];
WebAssembly.instantiate(bytes, {}).then(({ instance }) => {
  const e = instance.exports;
  e.generate(16, 0);
  let land = 0, tot = 0;
  for (let b = 0; b < e.nbiome(); b++) { const h = e.hist(b); tot += h; if (b > 1) land += h; }
  const fp = (BigInt(e.fp_hi() >>> 0) << 32n) | BigInt(e.fp_lo() >>> 0);
  console.log(`wasm: ${e.width()}x${e.height()} land=${Math.round(land * 100 / tot)}% fp=${fp}`);
  if (expected) {
    if (fp.toString() === expected) console.log('NATIVE == WASM DETERMINISM: PASS ✅');
    else { console.log(`NATIVE == WASM DETERMINISM: MISMATCH ❌ (native ${expected})`); process.exit(1); }
  }
});
