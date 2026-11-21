// Writes the puzzles of data/puzzles.csv into a page, replacing the block between
// "// BANK-START" and "// BANK-END". No dependencies (plain Node 18+).
//
//   node scripts/csv_to_page.mjs [data/puzzles.csv] [index.html]
//
// Run scripts/check_puzzles.mjs afterwards. The level order in the page follows the CSV.
import { readFileSync, writeFileSync } from 'node:fs';

const [csvPath = 'data/puzzles.csv', pagePath = 'index.html'] = process.argv.slice(2);
const SRC = { classic: 'F', new: 'N', 'ranking-list': 'R', generated: 'G' };

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

const [header, ...rows] = parseCsv(readFileSync(csvPath, 'utf8'));
const col = Object.fromEntries(header.map((h, i) => [h, i]));
const levels = new Map(), named = [];
for (const r of rows) {
  const level = r[col.level], p = r[col.puzzle], src = SRC[r[col.source]];
  const rating = Math.round(+r[col.rating] * 10);
  if (!/^[.0-9]{81}$/.test(p) || !src || !(rating >= 10 && rating <= 999)) {
    console.error(`bad row: ${r.join(',')}`);
    process.exit(1);
  }
  if (!levels.has(level)) levels.set(level, []);
  levels.get(level).push(`${p}${String(rating).padStart(3, '0')}${src}`);
  if (src === 'F') named.push(`  '${p}': '${r[col.name].replace(/\\/g, '\\\\').replace(/'/g, "\\'")}',`);
}

const js = [
  '// BANK-START (generated from data/puzzles.csv by scripts/build_banks.py or scripts/csv_to_page.mjs, do not edit by hand)',
  '/* Je Eintrag: 81 Zeichen Rätsel, 3 Ziffern Bewertung (114 = 11,4), 1 Zeichen Herkunft',
  '   (F bekanntes Rätsel, N neu gefunden, R Forums-Rangliste, G vorab erzeugt). */',
  'const NAMED = {', ...named, '};',
  'const BANK = {',
  ...[...levels].map(([level, lines]) => `  ${level}: \`\n${lines.join('\n')}\`,`),
  '};',
  '// BANK-END',
].join('\n');

const html = readFileSync(pagePath, 'utf8');
const start = html.indexOf('// BANK-START'), end = html.indexOf('// BANK-END');
if (start < 0 || end < start) {
  console.error(`${pagePath}: markers // BANK-START and // BANK-END not found`);
  process.exit(1);
}
writeFileSync(pagePath, html.slice(0, start) + js + html.slice(end + '// BANK-END'.length));
console.log(`${pagePath}: wrote ${rows.length} puzzles (${[...levels].map(([k, v]) => `${k} ${v.length}`).join(', ')})`);
