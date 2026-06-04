// build.js — embed noo.wasm (base64) into viewer.template.html -> noosphere.html (Node).
const fs = require('fs'); const path = require('path');
const wasm = fs.readFileSync(path.join(__dirname, 'noo.wasm'));
const tpl = fs.readFileSync(path.join(__dirname, 'viewer.template.html'), 'utf8');
const out = tpl.replace('__WASM_B64__', wasm.toString('base64'));
fs.writeFileSync(path.join(__dirname, 'noosphere.html'), out);
console.log(`wrote noosphere.html (${out.length} bytes; embeds ${wasm.length}-byte wasm)`);
