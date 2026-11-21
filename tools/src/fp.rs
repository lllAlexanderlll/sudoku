//! Isomorphism-invariant fingerprints. Two puzzles that are transformations of each other
//! (digit relabelling, row/column/band/stack permutation, transposition) always get the same
//! fingerprint. Different puzzles collide only rarely (then one is wrongly treated as known,
//! which is the safe direction for a novelty check).

use crate::grid::{bx, col, row, Puzzle};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

fn h<T: Hash>(x: &T) -> u64 {
    let mut s = DefaultHasher::new();
    x.hash(&mut s);
    s.finish()
}

fn transpose(p: &Puzzle) -> Puzzle {
    let mut q = [0u8; 81];
    for r in 0..9 { for c in 0..9 { q[c * 9 + r] = p[r * 9 + c]; } }
    q
}

/// Weisfeiler-Lehman style refinement over the clue graph, with per-cell base labels.
fn refine(p: &Puzzle, base: &[u64; 81]) -> u64 {
    let cells: Vec<usize> = (0..81).filter(|&i| p[i] != 0).collect();
    let mut lab = *base;
    for _round in 0..3 {
        let mut next = [0u64; 81];
        for &i in &cells {
            let mut rs: Vec<u64> = Vec::new();
            let mut cs: Vec<u64> = Vec::new();
            let mut bs: Vec<u64> = Vec::new();
            let mut ds: Vec<u64> = Vec::new();
            let mut band: Vec<u64> = Vec::new();
            let mut stack: Vec<u64> = Vec::new();
            for &j in &cells {
                if j == i { continue; }
                if row(j) == row(i) { rs.push(lab[j]); }
                if col(j) == col(i) { cs.push(lab[j]); }
                if bx(j) == bx(i) { bs.push(lab[j]); }
                if p[j] == p[i] { ds.push(lab[j]); }
                if row(j) / 3 == row(i) / 3 { band.push(lab[j]); }
                if col(j) / 3 == col(i) / 3 { stack.push(lab[j]); }
            }
            for v in [&mut rs, &mut cs, &mut bs, &mut ds, &mut band, &mut stack] { v.sort_unstable(); }
            next[i] = h(&(lab[i], rs, cs, bs, ds, band, stack));
        }
        lab = next;
    }
    let mut all: Vec<u64> = cells.iter().map(|&i| lab[i]).collect();
    all.sort_unstable();
    h(&all)
}

fn puzzle_fp_oriented(p: &Puzzle) -> u64 {
    let mut base = [0u64; 81];
    let mut rc = [0u8; 9];
    let mut cc = [0u8; 9];
    let mut bc = [0u8; 9];
    let mut dc = [0u8; 10];
    for i in 0..81 { if p[i] != 0 { rc[row(i)] += 1; cc[col(i)] += 1; bc[bx(i)] += 1; dc[p[i] as usize] += 1; } }
    for i in 0..81 {
        if p[i] != 0 { base[i] = h(&(rc[row(i)], cc[col(i)], bc[bx(i)], dc[p[i] as usize])); }
    }
    refine(p, &base)
}

pub fn puzzle_fp(p: &Puzzle) -> u64 {
    puzzle_fp_oriented(p).min(puzzle_fp_oriented(&transpose(p)))
}

/// Fingerprint of a full solution grid (used to keep one puzzle per solution grid family).
fn grid_fp_oriented(g: &Puzzle) -> u64 {
    // Base label: number of 4-cell deadly rectangles through the cell, and how many
    // mini-rows/mini-columns elsewhere in the band/stack share 2+ digits with the cell's own.
    let mini_row = |i: usize| -> u16 { let s = row(i) * 9 + (col(i) / 3) * 3; (0..3).fold(0, |a, k| a | 1 << g[s + k]) };
    let mini_col = |i: usize| -> u16 { let s = (row(i) / 3) * 27 + col(i); (0..3).fold(0, |a, k| a | 1 << g[s + 9 * k]) };
    let mut base = [0u64; 81];
    for i in 0..81 {
        let (r, c) = (row(i), col(i));
        let mut rect = 0u32;
        for r2 in 0..9 {
            if r2 == r { continue; }
            for c2 in 0..9 {
                if c2 == c { continue; }
                let same_band = r2 / 3 == r / 3;
                let same_stack = c2 / 3 == c / 3;
                if same_band == same_stack { continue; } // must span exactly two boxes
                if g[r * 9 + c] == g[r2 * 9 + c2] && g[r * 9 + c2] == g[r2 * 9 + c] { rect += 1; }
            }
        }
        let (mr, mc) = (mini_row(i), mini_col(i));
        let mut share_r = 0u32;
        let mut share_c = 0u32;
        for s in 0..3 {
            if s != c / 3 { let j = r * 9 + s * 3; if (mini_row(j) & mr).count_ones() >= 2 { share_r += 1; } }
            if s != r / 3 { let j = s * 27 + c; if (mini_col(j) & mc).count_ones() >= 2 { share_c += 1; } }
        }
        base[i] = h(&(rect, share_r, share_c));
    }
    refine(g, &base)
}

pub fn grid_fp(g: &Puzzle) -> u64 {
    grid_fp_oriented(g).min(grid_fp_oriented(&transpose(g)))
}
