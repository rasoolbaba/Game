// sim.mjs — pure JS reference using BigInt (exact integer / i64-faithful via masking).
// Same integer algorithm as the Rust and AssemblyScript ports. Prints a 64-bit FNV
// fingerprint (acc) of every output across all ticks + timing. Determinism is proven
// when acc matches across all three implementations for the same T.
import { LUT } from '../gen/lut.js';

const MASK64 = (1n << 64n) - 1n;
const u64 = (x) => x & MASK64;              // unsigned 64 bit-pattern (two's complement for negatives)
const i64 = (x) => BigInt.asIntN(64, x);    // signed 64
const FBITS = 16n, SCALE = 65536n;
const XMIN_FIXED = -524288n, XMAX_FIXED = 524288n, STEP = 2048n;
const lut = LUT.map(BigInt);

const fmul = (a, b) => i64((a * b) >> FBITS);   // Q16.16 multiply, arithmetic shift
function ftanh(x) {
  if (x <= XMIN_FIXED) return lut[0];
  if (x >= XMAX_FIXED) return lut[lut.length - 1];
  const pos = x - XMIN_FIXED;                    // >= 0
  const idx = pos / STEP;                         // truncates toward 0; pos>=0 → floor
  const frac = pos - idx * STEP;
  const ii = Number(idx);
  const y0 = lut[ii], y1 = lut[ii + 1];
  return i64(y0 + ((y1 - y0) * frac) / STEP);     // (y1-y0)>=0, frac>=0 → consistent
}

let st = 0n;
const next = () => { let x = st; x = u64(x ^ u64(x << 13n)); x = u64(x ^ (x >> 7n)); x = u64(x ^ u64(x << 17n)); st = x; return x; };
const randw = () => i64(next() % 131072n) - 65536n; // real ~[-1,1)

const IN = 8, HID = 16, OUT = 8;
const T = Number(process.argv[2] || 200000);
st = 0x9E3779B97F4A7C15n;
const W1 = [], b1 = [], W2 = [], b2 = [];
for (let j = 0; j < HID; j++) { const r = []; for (let i = 0; i < IN; i++) r.push(randw()); W1.push(r); b1.push(randw()); }
for (let k = 0; k < OUT; k++) { const r = []; for (let j = 0; j < HID; j++) r.push(randw()); W2.push(r); b2.push(randw()); }
let v = []; for (let i = 0; i < IN; i++) v.push(BigInt(i + 1) * SCALE / 10n);

let acc = 14695981039346656037n; const PRIME = 1099511628211n;
const t0 = process.hrtime.bigint();
for (let t = 0; t < T; t++) {
  const h = [];
  for (let j = 0; j < HID; j++) { let s = b1[j]; for (let i = 0; i < IN; i++) s = i64(s + fmul(W1[j][i], v[i])); h.push(ftanh(s)); }
  const o = [];
  for (let k = 0; k < OUT; k++) { let s = b2[k]; for (let j = 0; j < HID; j++) s = i64(s + fmul(W2[k][j], h[j])); o.push(ftanh(s)); }
  v = o;
  for (let k = 0; k < OUT; k++) acc = u64((acc ^ u64(o[k])) * PRIME);
}
const t1 = process.hrtime.bigint();
const ms = Number(t1 - t0) / 1e6;
console.log(JSON.stringify({ impl: 'js-bigint', T, acc: acc.toString(), v: v.map(String), ms: Math.round(ms), ticks_per_s: Math.round(T / (ms / 1000)) }));
