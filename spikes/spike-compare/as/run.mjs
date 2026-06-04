// Loader: instantiate the AssemblyScript WASM and time run(T). u64 return → BigInt.
import { run } from './build/sim.js';
const T = Number(process.argv[2] || 200000);
const t0 = process.hrtime.bigint();
const acc = run(T);
const t1 = process.hrtime.bigint();
const ms = Number(t1 - t0) / 1e6;
console.log(JSON.stringify({ impl: 'assemblyscript-wasm', T, acc: acc.toString(), ms: Math.round(ms), ticks_per_s: Math.round(T / (ms / 1000)) }));
