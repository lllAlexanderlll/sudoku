# Sudoku zum Vorlesen

An accessible sudoku page (`index.html`, German) for a blind solver: it picks a puzzle and
reads the givens aloud block by block, so the puzzle can be set up on a physical board.
Everything runs locally in one HTML file; no network access is needed.

* **[HANDOVER.md](HANDOVER.md):** for whoever maintains the live page. Covers the puzzle file,
  the changes compared with the original page, and how to bring them in.
* **[RATINGS.md](RATINGS.md):** what the ratings mean.
* **`data/puzzles.csv`:** all 3,100 embedded puzzles with solution, rating, source and name.

## Levels

| Level    | Sudoku Explainer rating | Source |
|----------|------------------------|--------|
| Leicht   | –                      | generated in the browser, singles only (about 38 clues) |
| Mittel   | –                      | generated in the browser, singles only (about 31 clues) |
| Schwer   | 2.6 – 4.4              | pre-generated and rated bank |
| Experte  | 4.5 – 8.9              | pre-generated and rated bank |
| Meister  | 9.0 – 11.3             | forum ranking list of the hardest sudokus (ph2010), named classics below 11.4, new finds |
| Extrem   | 11.5 – 11.9            | the 1,000 hardest distinct solution grids from the ranking list, plus the named classics |

From Schwer upwards the page draws from embedded banks, so there is no waiting on the client.
Every puzzle in a level comes up once before any repeats (a seeded shuffle; only the seed and
position are kept in `localStorage`). Each puzzle is also randomly relabelled, permuted and
possibly transposed before it is read, which keeps the difficulty identical.

The ratings are Sudoku Explainer (SE) ratings: the difficulty of the hardest single step needed.
They are announced with each puzzle ("Bewertung 11,5"). See [RATINGS.md](RATINGS.md) for how they
are computed and why 9.0 to 11.9 is a far bigger step than 2.6 to 4.4.

## How the banks are built

* **Generation (Schwer, Experte):** random solution grid, then clues removed in random order
  while the solution stays unique, which gives a random *minimal* puzzle.
  `tools/` produces about 6,000 per second per core.
* **Pre-filter:** a small human-style solver (singles, locked candidates, subsets, X-Wing,
  Swordfish, XY/XYZ-Wing) sorts candidates, so only promising ones go to the slow rater.
* **Rating:** [SukakuExplainer](https://github.com/SudokuMonster/SukakuExplainer) `serate`, the
  maintained batch version of Sudoku Explainer that the forum ratings use. It takes about 0.1–1 s
  per Schwer/Experte puzzle, but several minutes for an 11+ puzzle.
* **T&E depth (Berthier):** the number of nested levels of trial and error with singles that a
  puzzle needs. Depth 2 contains essentially all "potential hardest" puzzles (SE ≥ ~9); depth 3 is
  the post-2022 tridagon class. It takes milliseconds and is the main filter for hard candidates.
* **Vicinity search {-k+k}:** around top puzzles, remove k clues, find every way to add k clues
  that makes the solution unique again (by counting over all solutions of the reduced puzzle),
  minimise, keep T&E depth 2, and drop anything already known. Candidates are rated afterwards.
* **Novelty and diversity:** isomorphism-invariant fingerprints of puzzles and of solution grids.
  New finds are checked against about 3.4 million known hard puzzles. No two puzzles in any of
  the banks share a solution grid (the named classics excepted), because near-twins from vicinity
  searches would feel the same. New finds make up at most 40% of a bank.
* **Verification:** before writing, every puzzle is checked again for a unique solution and
  minimality. `scripts/check_puzzles.mjs` re-checks `data/puzzles.csv` and the page
  independently (plain Node, its own solver). `scripts/test_page.js` draws puzzles from every
  level in a headless browser and checks them against the banks.

## Findings

* **Ratings are consistent.** Re-rating 8 random ranking-list puzzles (10.4–11.8) with our
  SukakuExplainer build gave the published value 6 times and +0.1 twice.
* **Duplicate classic.** "eleven #1" and "eleven #3" on the original page were the same puzzle
  (checked with an exact isomorphism test over all 3.4 million transformations). There are 16
  distinct classics. AI Escargot (10.5), Platinum Blonde (10.6) and Arto Inkala's 2012 puzzle
  (10.6) rate below 11.4, so they now appear under Meister.
* **New puzzles.** The {-1+1} vicinity of the 208 ranking-list puzzles rated 11.7+ contains
  3,499 minimal T&E(2) puzzles that are in none of the collections used here.
  826 of them were rated: 4 rate 8.0, the rest 9.0–11.1, most of them 9.9–10.6. None reached 11.4.
  After removing puzzles whose solution grid is already used in a bank, 276 new finds
  (9.0–11.0) are in Meister.
  Puzzles with no T&E(1) eliminations at the start rate higher on average (about 10.3 against
  about 9.8), so they were rated first. Finding new 11.4+ puzzles needs far more compute (the forum
  has mined these neighbourhoods for years) and is the obvious next step: `{-2+2}` searches and
  rating many more candidates.
* **Random puzzles.** Among 200,000 random minimal puzzles none had T&E depth 2. About 2.6%
  rate 8.0 or more and about 0.14% reach 9.0 (estimated from 12,000 rated). That is why Meister
  and Extrem come mostly from the forum lists.

## Reproducing

```bash
cd tools && cargo build --release && cd ..
T=tools/target/release/sudoku-tools
$T gen 200000 20261121 > gen.txt                       # puzzle clues tier
# rate a selection with SukakuExplainer:
java -cp SukakuExplainer diuf.sudoku.test.serate --format="%g %r/%p/%d %T" --threads=4 \
     --input=selection.txt --output=generated.rated
# vicinity search around the top of the ranking list, excluding known puzzles:
$T pfp < all_known.txt | sort -u > known.fp
$T vicinity 1 known.fp < seeds.txt > candidates.txt     # puzzle depth te1-elims seed#
python3 scripts/build_banks.py --ph2010 Sudoku-classif/ph2010-3103972-TE2/PUZZLES \
     --rated generated.rated:G --rated new.rated:N --rated classics.rated:C
node scripts/check_puzzles.mjs data/puzzles.csv index.html
node scripts/test_page.js
```

`build_banks.py` writes `data/puzzles.csv` and the BANK block in `index.html`. After editing the
CSV by hand, `node scripts/csv_to_page.mjs` rewrites the BANK block from it.

## Sources and credits

* ph2010 "potential hardest" list (3.1 million puzzles rated 10.3–11.9), compiled by champagne
  and others on [enjoysudoku.com](http://forum.enjoysudoku.com/the-hardest-sudokus-new-thread-t6539.html),
  taken with SE ratings from [denis-berthier/Sudoku-classif](https://github.com/denis-berthier/Sudoku-classif) (GPL-3.0).
  The same repository provided the eleven, Paquita, Monhard and mith T&E(3) collections, and
  [tdoku](https://github.com/t-dillon/tdoku) the ph_1905 list; these were used for the novelty check.
* Ratings: [SukakuExplainer](https://github.com/SudokuMonster/SukakuExplainer) (LGPL-2.1),
  built from commit 362854e. It is not vendored here.
* T&E depth: Denis Berthier, *Pattern-Based Constraint Satisfaction and Logic Puzzles*.
