// Loader for the Rust→WASM build. Times the run() call; u64 param/return ↔ BigInt.
import { readFileSync } from 'node:fs';
const bytes = readFileSync(new URL('./sim_wasm.wasm', import.meta.url));
const { instance } = await WebAssembly.instantiate(bytes, {});
const run = instance.exports.run;
const T = BigInt(process.argv[2] || 200000);
const t0 = process.hrtime.bigint();
const acc = run(T);
const t1 = process.hrtime.bigint();
const ms = Number(t1 - t0) / 1e6;
// WASM i64 returns arrive as SIGNED BigInt; normalize to unsigned 64-bit for comparison.
const accU = BigInt.asUintN(64, acc);
console.log(JSON.stringify({ impl: 'rust-wasm', T: Number(T), acc: accU.toString(), ms: Math.round(ms), ticks_per_s: Math.round(Number(T) / (ms / 1000)) }));
