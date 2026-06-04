// tools/field-snapshot.js — REAL browser field-capture (Playwright/chromium), FRAME-HONEST.
// Loads each self-contained viewer, lets the deterministic sim run, drives the god-game like a
// real player (clicks = verses), then FREEZES the sim (overrides requestAnimationFrame so the
// rAF loop halts after the in-flight frame) and captures the screenshot AND the HUD stats from
// the SAME frozen frame — so the recorded numbers always match the pixels. Writes
// field-reports/<date>/ PNGs + a per-shot stats manifest. Literal screenshots, not a re-render.
//   run:  NODE_PATH="$(npm root -g)" node tools/field-snapshot.js [YYYY-MM-DD]
const path = require('path'); const fs = require('fs');
const { chromium } = require('playwright');

const DATE = process.argv[2] || new Date().toISOString().slice(0, 10);
const OUT = path.resolve('field-reports', DATE);
fs.mkdirSync(OUT, { recursive: true });

async function freeze(page) {                       // halt the sim so stats == pixels (frame-honest)
  await page.evaluate(() => { window.requestAnimationFrame = function () { return 0; }; });
  await page.waitForTimeout(120);                   // let the already-scheduled frame finish, then no more
}
async function readStats(page) {
  return await page.evaluate(() => {
    const pick = s => { const e = document.querySelector(s); return e ? e.innerText.replace(/\s+/g, ' ').trim() : null; };
    return { hud: pick('.hud') || pick('.stats') || pick('.bar'), body: document.body.innerText.replace(/\s+/g, ' ').trim().slice(0, 360) };
  });
}
async function shoot(page, fileBase) {              // freeze -> read stats -> screenshot (same frame)
  await freeze(page);
  const stats = await readStats(page);
  const file = fileBase + '.png';
  await page.screenshot({ path: path.join(OUT, file), fullPage: true });
  return { file, hud: stats.hud, body: stats.body };
}

(async () => {
  const browser = await chromium.launch({ args: ['--no-sandbox'] });
  const newPage = () => browser.newPage({ viewport: { width: 820, height: 1000 }, deviceScaleFactor: 2 });
  const manifest = [];

  for (const v of [
    { name: 'neural-life',  file: 'spikes/neural-life/web/neural-life.html',  wait: 3500 },
    { name: 'carbon-world', file: 'spikes/carbon-world/web/carbon-world.html', wait: 2200 },
    { name: 'noosphere',    file: 'spikes/noosphere/web/noosphere.html',       wait: 6000 },
  ]) {
    const page = await newPage(); const errors = []; page.on('pageerror', e => errors.push(e.message));
    await page.goto('file://' + path.resolve(v.file));
    await page.waitForTimeout(v.wait);
    const shot = await shoot(page, v.name);
    manifest.push({ name: v.name, file: v.file, shots: [shot], errors });
    await page.close(); console.log('shot: ' + v.name + '  hud="' + shot.hud + '"  errors=' + (errors.length ? errors.join('|') : 'none'));
  }

  // god-game: two independent deterministic loads -> a frame-honest "natural" and "after verses"
  const gfile = 'spikes/godgame/web/godgame.html'; const gerr = []; const gshots = [];
  {
    const page = await newPage(); page.on('pageerror', e => gerr.push(e.message));
    await page.goto('file://' + path.resolve(gfile)); await page.waitForTimeout(3500);
    gshots.push(await shoot(page, 'godgame')); await page.close();
  }
  {
    const page = await newPage(); page.on('pageerror', e => gerr.push(e.message));
    await page.goto('file://' + path.resolve(gfile)); await page.waitForTimeout(3500);
    const box = await page.locator('#c').boundingBox();
    await page.waitForTimeout(2500);                                  // accrue devotion
    for (const [fx, fy] of [[0.25, 0.35], [0.5, 0.4], [0.7, 0.55], [0.4, 0.65], [0.62, 0.28], [0.33, 0.55]]) {
      await page.mouse.click(box.x + box.width * fx, box.y + box.height * fy); await page.waitForTimeout(300);
    }
    await page.waitForTimeout(2800);                                  // let the verses spread
    gshots.push(await shoot(page, 'godgame-after-verses')); await page.close();
  }
  manifest.push({ name: 'godgame', file: gfile, shots: gshots, errors: gerr });
  console.log('shot: godgame x2  ' + gshots.map(s => '"' + s.hud + '"').join('  ') + '  errors=' + (gerr.length ? gerr.join('|') : 'none'));

  await browser.close();
  fs.writeFileSync(path.join(OUT, 'manifest.json'), JSON.stringify({ date: DATE, note: 'stats captured from the same frozen frame as each screenshot', viewers: manifest }, null, 2));
  console.log('wrote ' + path.relative(process.cwd(), OUT) + '/manifest.json');
})().catch(e => { console.error('FATAL ' + e.message); process.exit(1); });
