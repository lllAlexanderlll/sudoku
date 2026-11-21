# What the ratings mean

The page announces each puzzle's Sudoku Explainer rating ("Bewertung 11,5"). It is the scale the
puzzle community uses for the hardest sudokus. The numbers here were computed with
SukakuExplainer, the maintained version of Sudoku Explainer. The facts about chains below come
from its source code (`Chaining.getDifficulty`, `ChainingHint.getLengthDifficulty`).

## How a rating is computed

The rater solves the puzzle like a methodical human. At every step it looks for the
*easiest* technique that makes progress and applies it. Every technique has a fixed rating. The
puzzle's rating is the rating of the **hardest step on this easiest path**. So "11.5" means that
at some point no step easier than 11.5 exists.

Two consequences:

* The rating says how hard the hardest moment is, not how long the puzzle takes. Two
  puzzles rated 10.6 can need one hard step or twenty.
* It only knows its own list of techniques. Experts solve the very hardest puzzles with patterns
  the rater does not use (exocets, SK loops, multi-sector locked sets). For a solver who knows
  them, a high-rated puzzle can be easier than its number suggests.

## Up to about 8: one number per technique

Below about 8 the numbers are labels for named patterns of fixed size. They are ordered by
difficulty but are not a measure of effort. Approximate values from the original Sudoku
Explainer table (this build adds a few variants in between, such as 2.9, 4.1 and 4.3):

| Rating | Technique |
|--------|-----------|
| 1.0–1.5 | hidden single |
| 2.3 | naked single |
| 2.6 / 2.8 | pointing / claiming (locked candidates) |
| 3.0 / 3.4 | naked pair / hidden pair |
| 3.2 / 3.8 | X-Wing / Swordfish |
| 3.6 / 4.0 | naked triple / hidden triple |
| 4.2 / 4.4 | XY-Wing / XYZ-Wing |
| 4.5–5.0 | unique rectangles and loops |
| 5.0 / 5.2 / 5.4 | naked quad / Jellyfish / hidden quad |
| 5.6–6.0 | BUG (bivalue universal grave) |
| 6.2 / 7.5 | aligned pair / triple exclusion |
| 6.5–7.5 | X-cycles and simple forcing chains |
| 7.5–8.0 | Nishio forcing chains |

Schwer (2.6–4.4) therefore means "pairs, triples, X-Wing, Swordfish, XY/XYZ-Wing". Each of
these is a small pattern of a few cells that can be spotted directly. Going from 3.0 to 4.0 adds
one pattern to look for. It does not multiply the work.

## From 8.0: chains, nesting and length

From 8.0 upward every technique is a **forcing chain**: "if this cell were 5, then … then … and
that leads to a contradiction". The rating is a base value for the *kind* of chain plus a bonus
for its *size*.

Base value by kind of chain:

| Base | Kind of chain |
|------|---------------|
| 8.0 | multiple forcing chains (several chains from one cell or one house) |
| 8.5 | dynamic forcing chains (a chain may use the results of earlier links) |
| 9.0 | dynamic forcing chains "+": the chain may also use basic techniques along the way |
| 9.5 | nested forcing chains, level 2: a link of the chain is itself proved by a forcing chain |
| 10.0 | nested, level 3 |
| 10.5 | nested, level 4 |

Size bonus: +0.1 each time the chain (all nested sub-chains counted) grows past
4, 6, 8, 12, 16, 24, 32, 48, 64, 96, 128, 192, 256, 384, 512 … links. The steps alternate
between ×1.5 and ×1.33, so:

* **+0.1 ≈ 1.4 times as many links**
* **+0.2 = twice as many links**
* **+0.5 ≈ six times as many links, or one more level of nesting**

That is why a small difference matters so much at the top:

| Rating | What the hardest step needs (one of) |
|--------|--------------------------------------|
| 9.0 | a dynamic "+" chain of up to 4 links, or a dynamic chain of about 20 links |
| 10.0 | chains nested three deep of up to 4 links, nested two deep of about 20, or a dynamic "+" chain of about 100–130 |
| 11.3 | chains nested four deep with about 50–64 links in total, or three deep with about 260–380 |
| 11.9 | chains nested four deep with about 400–500 links in total |

From 9.0 to 11.9 the hardest step grows more than a hundredfold. From 2.6 to 4.4 it only
moves from one small pattern to a slightly larger one.

## How rare these puzzles are

* Among 200,000 random minimal puzzles, about 2.6% rate 8.0 or more and about 0.14% reach 9.0
  (estimated from 12,000 that were rated).
* Almost all Meister puzzles and all Extrem puzzles need **two nested levels of trial and error**
  (T&E depth 2, Denis Berthier's measure). Assume a cell value and follow only singles; at some
  point you must assume a second value inside the first one. None of the 200,000 random puzzles
  needed that.
* The Extrem bank is the top of the forum's ranking list of about 3.1 million such puzzles,
  collected over many years. Only about 200 known puzzles rate 11.7 or more.

## Trust in the numbers

Re-rating 8 random puzzles (10.4–11.8) from the forum list with this build gave the published
value 6 times and 0.1 more twice. Ratings near the top can differ by 0.1 between Sudoku Explainer
versions. That is normal and does not change which level a puzzle belongs to.
