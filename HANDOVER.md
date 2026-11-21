# Handover: new puzzles for the Sudoku page

For Simon, who maintains the website. This covers what changed, which files to use, and how
to bring the changes into the live page.

## In short

* `data/puzzles.csv`: **3,100 rated puzzles with solutions.** Schwer 500, Experte 800, the new
  level Meister 800, Extrem 1,000. Every puzzle was checked: exactly one solution, and no clue
  can be removed.
* `index.html`: the original page with all changes already made and the puzzles embedded.
* `scripts/check_puzzles.mjs` and `scripts/csv_to_page.mjs`: plain Node 18+, nothing to install.
* `RATINGS.md`: what the ratings mean, and why 9.0 vs 11.9 is a huge step while 2.6 vs 4.4 is not.

## The puzzle file

`data/puzzles.csv` (UTF-8, comma-separated, quoted where needed):

| Column | Example | Meaning |
|--------|---------|---------|
| level | `extrem` | `schwer`, `experte`, `meister`, `extrem` |
| puzzle | `98.7.....7.....6...` | 81 characters, row by row, `.` = empty |
| solution | `987654321246...` | 81 digits |
| rating | `11.9` | Sudoku Explainer rating, one decimal |
| source | `classic` | `classic` named famous puzzle, `new` found in this project, `ranking-list` forum list of the hardest sudokus, `generated` generated here |
| name | `„Champagne Dry“` | classics: German phrase as used in "Es beruht auf …"; ranking list: the forum's ID (not shown on the page); otherwise empty |

Rating ranges per level: Schwer 2.6–4.4, Experte 4.5–8.9, Meister 9.0–11.3, Extrem 11.5–11.9.

## Bringing it into the live page

**Option A, if the live page is still the version Alexander sent (the first commit of this
repository, `69ba738`):** replace it with `index.html`. To check, compare the live file with
`git show 69ba738:index.html`; if they are identical, this is the whole job.

**Option B, if the live page has changed since:** port the changes listed below. The prompt at
the end of this file is written so that Claude can do it.

## Changes compared with the original page

All changes are in the `<script>` part, except the level list in the HTML.

1. **Puzzles come from embedded banks instead of being generated in the browser (Schwer to
   Extrem).** The block between `// BANK-START` and `// BANK-END` holds `NAMED` (names of the
   classics) and `BANK` (one string per level, one line per puzzle: 81 characters, a 3-digit
   rating such as `115` for 11.5, a 1-letter source `F`/`N`/`R`/`G`). It replaces the
   `FAMOUS` list.
   *Why:* the old Schwer and Experte puzzles took up to 12 seconds to generate. Their difficulty
   was only "not solvable with simple steps", which can mean anything from 2.6 to 11. Now
   every puzzle is rated and appears instantly.
2. **New level "Meister" (9.0–11.3)** between Experte and Extrem: a new radio button and an
   entry in `LEVELS`. The old Extrem (17 classics) mixed 10.5 with 11.9.
3. **No repeats per level:** `nextFromBank(level)` replaces `nextFamous()`. It stores only a
   random seed and a position in `localStorage` under `sudoku-ansage-stapel-v2-<level>`, and
   rebuilds the shuffled order from the seed. The old key `sudoku-ansage-extrem-stapel-v1` is
   no longer used and can be left alone. The saved game (`sudoku-ansage-game-v2`) and the
   settings are unchanged.
4. **The rating is announced:** the "Es beruht auf …" text now says where the puzzle comes from
   and ends with "(Bewertung 11,5)".
5. **Level descriptions** show the rating range and the number of puzzles, filled in from the
   bank at start (`data-range`, `data-bank`). They are no longer hard-coded.
6. **"Bitte warten" only for Leicht and Mittel**, the levels that are still generated in the
   browser.
7. **Removed code:** `eliminate()` and the multi-level `logicSolve()`, which only the old
   Schwer/Experte generation used. `logicSolve()` now checks singles only, for Leicht/Mittel.
8. **Duplicate removed:** "eleven #1" and "eleven #3" were the same puzzle (checked with an
   exact isomorphism test). There are 16 classics. Three of them (AI Escargot, Platinum Blonde,
   Arto Inkala 2012) rate 10.5–10.6 and are now under Meister.

The page grows from 95 KB to 352 KB. That is fine to load, also on a phone.

## Recommendations

**Keep the shuffling.** There are two kinds, and both were already in the original page:

* *Shuffled order without repeats* (change 3): keep it, or Alexander's mother would get
  repeats or always the same order.
* *Transforming each puzzle before reading it* (`transform()`: relabel digits, swap rows,
  columns and bands, maybe mirror). The original did this for Extrem; now it applies to all bank
  levels. Keep it. It costs nothing, it leaves the difficulty unchanged, and it means a famous
  puzzle cannot be recognised from its givens. The page then works out the solution of the
  transformed puzzle with `solve()`, which takes under 50 ms even for the hardest puzzles
  (measured on all Meister and Extrem puzzles). Dropping it would save only about 15 lines.

**Embed the puzzles; don't load the CSV with `fetch()`.** Embedding keeps the page a single file
that also works offline and when opened directly from disk (`file://`), where `fetch()` fails.
Use the CSV as the source and regenerate the embedded block from it (see below).

**No verification in the page.** Every puzzle was verified when the banks were built, and the
page computes the solution itself anyway. But **run the check whenever the puzzle data or the
code that embeds it changes.** One wrong character makes a puzzle unsolvable or ambiguous, and
she would only notice after a long solve:

```bash
node scripts/check_puzzles.mjs data/puzzles.csv index.html
```

It checks every puzzle in the CSV (one solution, matches the solution column, minimal, rating
fits the level, no duplicates) and that the page holds exactly the CSV's puzzles with the same
levels and ratings. It takes about 2 minutes and ends with `ALL OK`. If the live page embeds
the puzzles differently, ask Claude to adapt the page part of the script.

## Changing the puzzles later

* **Remove or move puzzles:** edit `data/puzzles.csv`, then:

```bash
node scripts/csv_to_page.mjs data/puzzles.csv index.html
```

```bash
node scripts/check_puzzles.mjs data/puzzles.csv index.html
```

* **Add puzzles:** they need a Sudoku Explainer rating. See `README.md`, "Reproducing", for
  how the banks were built and rated.

## Prompt for Claude (Option B)

> In this repository is our accessible sudoku page. I want to bring in the new puzzle banks from
> https://github.com/lllAlexanderlll/sudoku. Read its HANDOVER.md, compare its first commit's
> `index.html` (the version we had) with its current `index.html`, and apply the same changes
> to our page: the BANK block, `nextFromBank`, the Meister level, the rating in the "Es beruht
> auf" text, the level descriptions, the "Bitte warten" change, and the removed code. Keep
> everything else we changed since then. Keep `transform()` and the no-repeat order. Generate the
> BANK block with `node scripts/csv_to_page.mjs data/puzzles.csv <our page>`, not by hand. Then
> run `node scripts/check_puzzles.mjs data/puzzles.csv <our page>`. Open the page and check that
> all six levels start a puzzle without errors in the console.
