// build.js — embed god.wasm (base64) into viewer.template.html -> godgame.html (Node).
const fs = require('fs'); const path = require('path');
const wasm = fs.readFileSync(path.join(__dirname, 'god.wasm'));
const tpl = fs.readFileSync(path.join(__dirname, 'viewer.template.html'), 'utf8');
fs.writeFileSync(path.join(__dirname, 'godgame.html'), tpl.replace('__WASM_B64__', wasm.toString('base64')));
console.log(`wrote godgame.html (embeds ${wasm.length}-byte wasm)`);
