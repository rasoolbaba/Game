// verify.js — headless cross-runtime determinism check (Node).
// Instantiates life.wasm, runs the canonical scenario, and compares the WASM fingerprint
// against the NATIVE fingerprint passed as argv[2]. Proves: same Rust core, native == wasm.
const fs = require('fs');
const path = require('path');
const bytes = fs.readFileSync(path.join(__dirname, 'life.wasm'));
const expected = process.argv[2]; // native fingerprint (decimal string)
WebAssembly.instantiate(bytes, {}).then(({ instance }) => {
  const e = instance.exports;
  e.init(12345, 0);
  for (let t = 0; t < 500; t++) e.step();
  const lo = BigInt(e.fp_lo() >>> 0);
  const hi = BigInt(e.fp_hi() >>> 0);
  const fp = (hi << 32n) | lo;
  console.log(`wasm:   pop=${e.pop()} peak=? births=${e.births()} deaths=${e.deaths()} eaten=${e.eaten()} fp=${fp}`);
  if (expected) {
    if (fp.toString() === expected) console.log('NATIVE == WASM DETERMINISM: PASS ✅');
    else { console.log(`NATIVE == WASM DETERMINISM: MISMATCH ❌ (native ${expected})`); process.exit(1); }
  }
});
