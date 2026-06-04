// build.js — assemble the self-contained viewer (Node).
// Embeds life.wasm as base64 into viewer.template.html -> neural-life.html, so the viewer
// opens by double-click (file://) with no server and no separate .wasm fetch.
const fs = require('fs');
const path = require('path');
const wasm = fs.readFileSync(path.join(__dirname, 'life.wasm'));
const tpl = fs.readFileSync(path.join(__dirname, 'viewer.template.html'), 'utf8');
const out = tpl.replace('__WASM_B64__', wasm.toString('base64'));
fs.writeFileSync(path.join(__dirname, 'neural-life.html'), out);
console.log(`wrote neural-life.html (${out.length} bytes; embeds ${wasm.length}-byte wasm)`);
