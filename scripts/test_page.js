// Browser test for index.html: generates puzzles on every level and checks them.
// Usage: node scripts/test_page.js [puzzles-per-bank-level]
// Needs Playwright with Chromium and the release build of tools/ (cargo build --release).
const { chromium } = require('playwright');
const { execFileSync } = require('child_process');
const fs = require('fs');
const path = require('path');

const ROOT = path.dirname(__dirname);
const TOOL = path.join(ROOT, 'tools/target/release/sudoku-tools');
const N = +(process.argv[2] || 25);
const tool = (args, lines) => execFileSync(TOOL, args, { input: lines.join('\n') + '\n' }).toString().trim().split('\n').map(l => l.split(' '));
const toStr = a => a.map(v => (v ? String(v) : '.')).join('');

let failures = 0;
const fail = msg => { failures++; console.log('FAIL ' + msg); };

(async () => {
  const browser = await chromium.launch();
  const page = await browser.newPage();
  const errors = [];
  page.on('pageerror', e => errors.push(e.message));
  page.on('console', m => { if (m.type() === 'error') errors.push(m.text()); });
  page.on('dialog', d => d.accept());
  await page.goto('file://' + path.join(ROOT, 'index.html'));

  const texts = await page.$$eval('label.level', ls => ls.map(l => l.textContent));
  console.log('levels:\n  ' + texts.join('\n  '));
  if (texts.length !== 6) fail('expected 6 levels');

  async function generateOnce(level) {
    await page.check(`input[name=level][value=${level}]`);
    await page.click('#btn-new');
    await page.waitForFunction(() => !document.querySelector('#btn-new').disabled, null, { timeout: 30000 });
    return page.evaluate(() => JSON.parse(localStorage.getItem('sudoku-ansage-game-v2')));
  }

  for (const level of ['leicht', 'mittel']) {
    const S = await generateOnce(level);
    const [row] = tool(['check'], [toStr(S.puzzle)]);
    if (S.level !== level || row[1] !== '1') fail(`${level}: not unique`);
    console.log(`${level}: ${row[3]} clues, unique`);
  }

  for (const level of ['schwer', 'experte', 'meister', 'extrem']) {
    const bank = fs.readFileSync(path.join(ROOT, `data/${level}.tsv`), 'utf8').split('\n').filter(l => l && !l.startsWith('#')).map(l => l.split('\t')[0]);
    const bankFp = new Set(tool(['pfp'], bank).map(r => r[0]));
    const got = [];
    for (let i = 0; i < N; i++) {
      const S = await generateOnce(level);
      if (S.level !== level) fail(`${level}: wrong level ${S.level}`);
      if (!/Bewertung \d+,\d/.test(S.base)) fail(`${level}: no rating in "${S.base}"`);
      const sol = toStr(S.solution);
      got.push({ p: toStr(S.puzzle), base: S.base, sol });
    }
    const checks = tool(['check'], got.map(g => g.p));
    const fps = tool(['pfp'], got.map(g => g.p)).map(r => r[0]);
    checks.forEach((r, i) => {
      if (r[1] !== '1' || r[2] !== '1') fail(`${level}: puzzle not unique/minimal ${r[0]}`);
      if (r[4] !== got[i].sol) fail(`${level}: page solution differs from verified solution`);
      if (!bankFp.has(fps[i])) fail(`${level}: puzzle not from its bank ${r[0]}`);
    });
    if (new Set(fps).size !== fps.length) fail(`${level}: a puzzle repeated within ${N} draws`);
    console.log(`${level}: ${N} draws, ${new Set(fps).size} distinct, e.g. "${got[0].base}"`);
  }

  if (errors.length) fail('page errors: ' + errors.join(' | '));
  await browser.close();
  console.log(failures ? `${failures} FAILURES` : 'ALL OK');
  process.exit(failures ? 1 : 0);
})();
