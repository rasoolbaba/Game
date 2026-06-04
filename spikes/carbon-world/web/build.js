// build.js — embed world.wasm (base64) into viewer.template.html -> carbon-world.html (Node).
// Produces a single self-contained file that opens by double-click (file://), no server.
const fs = require('fs');
const path = require('path');
const wasm = fs.readFileSync(path.join(__dirname, 'world.wasm'));
const tpl = fs.readFileSync(path.join(__dirname, 'viewer.template.html'), 'utf8');
const out = tpl.replace('__WASM_B64__', wasm.toString('base64'));
fs.writeFileSync(path.join(__dirname, 'carbon-world.html'), out);
console.log(`wrote carbon-world.html (${out.length} bytes; embeds ${wasm.length}-byte wasm)`);
