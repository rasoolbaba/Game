// sim.ts — AssemblyScript port (compiles to WASM). Same integer algorithm as JS & Rust.
// Flat StaticArray<i64> buffers (no nested arrays) for speed + simple stub runtime.
import { LUT } from "../gen/lut";

const FBITS: i64 = 16;
const SCALE: i64 = 65536;
const XMIN_FIXED: i64 = -524288;
const XMAX_FIXED: i64 = 524288;
const STEP: i64 = 2048;

// @ts-ignore: i64 arithmetic
function fmul(a: i64, b: i64): i64 { return (a * b) >> FBITS; }

function ftanh(x: i64): i64 {
  if (x <= XMIN_FIXED) return LUT[0];
  if (x >= XMAX_FIXED) return LUT[LUT.length - 1];
  let pos: i64 = x - XMIN_FIXED;
  let idx: i32 = i32(pos / STEP);
  let frac: i64 = pos - i64(idx) * STEP;
  let y0: i64 = LUT[idx];
  let y1: i64 = LUT[idx + 1];
  return y0 + ((y1 - y0) * frac) / STEP;
}

let st: u64 = 0;
function nextr(): u64 { let x = st; x ^= x << 13; x ^= x >> 7; x ^= x << 17; st = x; return x; }
function randw(): i64 { return i64(nextr() % 131072) - 65536; }

// NOTE (auditor AUDIT-0004): T is i32 → caps at ~2.1e9 ticks. Fine for spike scale;
// widen to i64 if ever needed. JS/Rust ports use Number/u64 respectively.
export function run(T: i32): u64 {
  const IN = 8, HID = 16, OUT = 8;
  st = 0x9E3779B97F4A7C15;
  let w1 = new StaticArray<i64>(HID * IN);
  let b1 = new StaticArray<i64>(HID);
  let w2 = new StaticArray<i64>(OUT * HID);
  let b2 = new StaticArray<i64>(OUT);
  for (let j = 0; j < HID; j++) { for (let i = 0; i < IN; i++) { w1[j * IN + i] = randw(); } b1[j] = randw(); }
  for (let k = 0; k < OUT; k++) { for (let j = 0; j < HID; j++) { w2[k * HID + j] = randw(); } b2[k] = randw(); }

  let v = new StaticArray<i64>(IN);
  for (let i = 0; i < IN; i++) { v[i] = (i64(i) + 1) * SCALE / 10; }
  let h = new StaticArray<i64>(HID);
  let o = new StaticArray<i64>(OUT);

  let acc: u64 = 14695981039346656037;
  const prime: u64 = 1099511628211;
  for (let t = 0; t < T; t++) {
    for (let j = 0; j < HID; j++) { let s: i64 = b1[j]; for (let i = 0; i < IN; i++) { s += fmul(w1[j * IN + i], v[i]); } h[j] = ftanh(s); }
    for (let k = 0; k < OUT; k++) { let s: i64 = b2[k]; for (let j = 0; j < HID; j++) { s += fmul(w2[k * HID + j], h[j]); } o[k] = ftanh(s); }
    for (let i = 0; i < IN; i++) { v[i] = o[i]; }
    for (let k = 0; k < OUT; k++) { acc = (acc ^ u64(o[k])) * prime; }
  }
  return acc;
}
