// tools/field-snapshot.js — REAL browser field-capture (Playwright/chromium).
// Loads each self-contained viewer, lets the deterministic sim run, drives the god-game like a
// real player (clicks = verses), then writes full-page PNGs + a stats manifest into
// field-reports/<date>/. These are literal screenshots of the actual game, not a re-render.
//   run:  NODE_PATH="$(npm root -g)" node tools/field-snapshot.js [YYYY-MM-DD]
const path = require('path'); const fs = require('fs');
const { chromium } = require('playwright');

const DATE = process.argv[2] || new Date().toISOString().slice(0, 10);
const OUT = path.resolve('field-reports', DATE);
fs.mkdirSync(OUT, { recursive: true });

const VIEWERS = [
  { name: 'neural-life',  file: 'spikes/neural-life/web/neural-life.html',  wait: 3500 },
  { name: 'carbon-world', file: 'spikes/carbon-world/web/carbon-world.html', wait: 2500 },
  { name: 'noosphere',    file: 'spikes/noosphere/web/noosphere.html',       wait: 6000 },
  { name: 'godgame',      file: 'spikes/godgame/web/godgame.html',           wait: 3500, interact: 'god' },
];

(async () => {
  const browser = await chromium.launch({ args: ['--no-sandbox'] });
  const manifest = [];
  for (const v of VIEWERS) {
    const page = await browser.newPage({ viewport: { width: 820, height: 1000 }, deviceScaleFactor: 2 });
    const errors = []; page.on('pageerror', e => errors.push(e.message));
    await page.goto('file://' + path.resolve(v.file));
    await page.waitForTimeout(v.wait);
    const shots = [];

    const p0 = path.join(OUT, v.name + '.png');
    await page.screenshot({ path: p0, fullPage: true }); shots.push(path.basename(p0));

    if (v.interact === 'god') {                          // play it: build devotion, then cast verses
      const box = await page.locator('#c').boundingBox();
      await page.waitForTimeout(2500);
      const pts = [[0.25, 0.35], [0.5, 0.4], [0.7, 0.55], [0.4, 0.65], [0.62, 0.28], [0.33, 0.55]];
      for (const [fx, fy] of pts) { await page.mouse.click(box.x + box.width * fx, box.y + box.height * fy); await page.waitForTimeout(300); }
      await page.waitForTimeout(2800);
      const p1 = path.join(OUT, v.name + '-after-verses.png');
      await page.screenshot({ path: p1, fullPage: true }); shots.push(path.basename(p1));
    }

    const stats = await page.evaluate(() => {
      const pick = s => { const e = document.querySelector(s); return e ? e.innerText.replace(/\s+/g, ' ').trim() : null; };
      return { hud: pick('.hud') || pick('.stats') || pick('.bar'), body: document.body.innerText.replace(/\s+/g, ' ').trim().slice(0, 500) };
    });
    manifest.push({ name: v.name, file: v.file, shots, stats, errors });
    await page.close();
    console.log('shot: ' + v.name + '  errors=' + (errors.length ? errors.join('|') : 'none'));
  }
  await browser.close();
  fs.writeFileSync(path.join(OUT, 'manifest.json'), JSON.stringify({ date: DATE, viewers: manifest }, null, 2));
  console.log('wrote ' + path.relative(process.cwd(), OUT) + '/manifest.json');
})().catch(e => { console.error('FATAL ' + e.message); process.exit(1); });
