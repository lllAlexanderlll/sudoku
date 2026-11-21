// Checks data/puzzles.csv, and optionally the puzzles embedded in a page, without any
// dependencies (plain Node 18+).
//
//   node scripts/check_puzzles.mjs                      # check data/puzzles.csv
//   node scripts/check_puzzles.mjs data/puzzles.csv index.html
//
// For every puzzle: 81 characters, exactly one solution, the solution column is that
// solution, no clue can be removed (minimal), the rating fits the level, no duplicates.
// With a page: every puzzle in the page's BANK block is in the CSV with the same level and
// rating, and every CSV puzzle is in the page. Exits with code 1 on any problem.
import { readFileSync } from 'node:fs';

const LEVELS = { schwer: [2.6, 4.4], experte: [4.5, 8.9], meister: [9.0, 11.3], extrem: [11.4, 11.9] };

// --- a small, fast solver: bitmasks, naked singles, fewest-candidates-first branching ---
const PEERS = [...Array(81).keys()].map(i => {
  const r = (i / 9) | 0, c = i % 9, b = 3 * ((r / 3) | 0) + ((c / 3) | 0), s = new Set();
  for (let j = 0; j < 81; j++) {
    const rj = (j / 9) | 0, cj = j % 9, bj = 3 * ((rj / 3) | 0) + ((cj / 3) | 0);
    if (j !== i && (rj === r || cj === c || bj === b)) s.add(j);
  }
  return [...s];
});

/** Counts solutions of g (array of 81 digits, 0 = empty) up to limit; returns {count, solution}. */
function countSolutions(g, limit) {
  const a = g.slice();
  for (let i = 0; i < 81; i++) {
    if (a[i] && PEERS[i].some(p => a[p] === a[i])) return { count: 0, solution: null };
  }
  let count = 0, solution = null;
  (function rec() {
    let best = -1, bestMask = 0, bestN = 10;
    for (let i = 0; i < 81; i++) {
      if (a[i]) continue;
      let m = 0x3fe;
      for (const p of PEERS[i]) if (a[p]) m &= ~(1 << a[p]);
      const n = popcount(m);
      if (n < bestN) { best = i; bestMask = m; bestN = n; if (n <= 1) break; }
    }
    if (best < 0) { count++; if (!solution) solution = a.slice(); return; }
    for (let d = 1; d <= 9 && count < limit; d++) {
      if (bestMask & (1 << d)) { a[best] = d; rec(); a[best] = 0; }
    }
  })();
  return { count, solution };
}
const popcount = m => { let n = 0; while (m) { m &= m - 1; n++; } return n; };
const parse = s => [...s].map(ch => (ch === '.' || ch === '0' ? 0 : +ch));

// --- CSV with quoted fields ---
function parseCsv(text) {
  const rows = [];
  let row = [], field = '', q = false;
  for (let i = 0; i < text.length; i++) {
    const ch = text[i];
    if (q) {
      if (ch === '"' && text[i + 1] === '"') { field += '"'; i++; }
      else if (ch === '"') q = false;
      else field += ch;
    } else if (ch === '"') q = true;
    else if (ch === ',') { row.push(field); field = ''; }
    else if (ch === '\n' || ch === '\r') {
      if (ch === '\r' && text[i + 1] === '\n') i++;
      row.push(field); rows.push(row); row = []; field = '';
    } else field += ch;
  }
  if (field || row.length) { row.push(field); rows.push(row); }
  return rows.filter(r => r.length > 1);
}

const [csvPath = 'data/puzzles.csv', pagePath] = process.argv.slice(2);
const [header, ...rows] = parseCsv(readFileSync(csvPath, 'utf8'));
const col = Object.fromEntries(header.map((h, i) => [h, i]));
let problems = 0;
const problem = msg => { problems++; if (problems <= 20) console.log('PROBLEM ' + msg); };

const seen = new Map(), perLevel = {};
const t0 = Date.now();
rows.forEach((r, n) => {
  const line = n + 2, level = r[col.level], p = r[col.puzzle], sol = r[col.solution], rating = +r[col.rating];
  perLevel[level] = (perLevel[level] || 0) + 1;
  if (!/^[.0-9]{81}$/.test(p)) return problem(`line ${line}: puzzle is not 81 characters of 1-9 and .`);
  if (seen.has(p)) problem(`line ${line}: same puzzle as line ${seen.get(p)}`);
  seen.set(p, line);
  const range = LEVELS[level];
  if (!range) problem(`line ${line}: unknown level ${level}`);
  else if (!(rating >= range[0] - 1e-9 && rating <= range[1] + 1e-9)) problem(`line ${line}: rating ${rating} outside ${level} ${range.join('-')}`);
  const g = parse(p), res = countSolutions(g, 2);
  if (res.count !== 1) return problem(`line ${line}: ${res.count === 0 ? 'no solution' : 'more than one solution'}`);
  if (res.solution.join('') !== sol) problem(`line ${line}: solution column differs from the actual solution`);
  for (let i = 0; i < 81; i++) {
    if (!g[i]) continue;
    const d = g[i]; g[i] = 0;
    const unique = countSolutions(g, 2).count === 1;
    g[i] = d;
    if (unique) { problem(`line ${line}: not minimal (clue at row ${((i / 9) | 0) + 1}, column ${(i % 9) + 1} can be removed)`); break; }
  }
});
console.log(`${csvPath}: ${rows.length} puzzles (${Object.entries(perLevel).map(([k, v]) => `${k} ${v}`).join(', ')}), checked in ${((Date.now() - t0) / 1000).toFixed(1)} s`);

if (pagePath) {
  // BANK block of index.html: per level a template string of lines "<81 chars><3-digit rating><source>"
  const html = readFileSync(pagePath, 'utf8');
  const block = html.match(/const BANK = \{([\s\S]*?)\n\};/);
  if (!block) problem(`${pagePath}: no "const BANK = {...};" block found`);
  else {
    const inPage = new Set();
    for (const m of block[1].matchAll(/(\w+): `([^`]*)`/g)) {
      for (const e of m[2].trim().split('\n')) {
        const p = e.slice(0, 81), rating = +e.slice(81, 84) / 10, line = seen.get(p);
        inPage.add(p);
        if (!line) { problem(`${pagePath}: ${m[1]} puzzle ${p} is not in ${csvPath}`); continue; }
        const r = rows[line - 2];
        if (r[col.level] !== m[1] || Math.abs(+r[col.rating] - rating) > 1e-9) problem(`${pagePath}: ${p} is ${m[1]} ${rating} in the page but ${r[col.level]} ${r[col.rating]} in ${csvPath}`);
      }
    }
    const missing = [...seen.keys()].filter(p => !inPage.has(p));
    if (missing.length) problem(`${pagePath}: ${missing.length} puzzles of ${csvPath} are missing in the page, e.g. ${missing[0]}`);
    console.log(`${pagePath}: ${inPage.size} puzzles in the BANK block`);
  }
}
console.log(problems ? `${problems} PROBLEMS` : 'ALL OK');
process.exit(problems ? 1 : 0);
