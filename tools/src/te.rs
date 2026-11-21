//! T&E(n) procedure (Berthier): trial and error with basic singles propagation.
//! A candidate is eliminated if assuming it leads, with T&E(n-1), to a contradiction.
//! The T&E depth of a puzzle is the smallest n for which T&E(n) solves it.
//! Almost all puzzles have depth <= 1; depth 2 are the "potential hardest" (SER ~9-11.9),
//! depth 3 is the 2022+ tridagon class.

use crate::grid::{Puzzle, State};

#[derive(PartialEq, Clone, Copy, Debug)]
pub enum Status { Contradiction, Solved, Stuck }

fn base(s: &mut State) -> Status {
    if !s.propagate() { Status::Contradiction } else if s.solved() { Status::Solved } else { Status::Stuck }
}

/// Runs T&E(k) on s to its fixpoint. `elims` counts eliminations made at this level.
pub fn te(s: &mut State, k: u32, elims: &mut u32) -> Status {
    let st = base(s);
    if k == 0 || st != Status::Stuck { return st; }
    loop {
        let mut progress = false;
        for i in 0..81 {
            if s.v[i] != 0 { continue; }
            let mut m = s.c[i];
            while m != 0 {
                let b = m & m.wrapping_neg();
                m ^= b;
                if s.v[i] != 0 || s.c[i] & b == 0 { continue; }
                let d = b.trailing_zeros() as u8 + 1;
                let mut t = s.clone();
                let mut dummy = 0;
                let contra = !t.assign(i, d) || te(&mut t, k - 1, &mut dummy) == Status::Contradiction;
                if contra {
                    *elims += 1;
                    progress = true;
                    if !s.eliminate(i, d) { return Status::Contradiction; }
                    match base(s) {
                        Status::Stuck => {}
                        other => return other,
                    }
                }
            }
        }
        if !progress { return Status::Stuck; }
    }
}

/// Returns (depth, elims at depth-1 level from the start state). Depth is capped: returns
/// `max+1` if T&E(max) does not solve it.
pub fn depth(p: &Puzzle, max: u32) -> u32 {
    let s0 = match State::from_puzzle(p) { Some(s) => s, None => return 99 };
    for k in 0..=max {
        let mut s = s0.clone();
        let mut e = 0;
        if te(&mut s, k, &mut e) == Status::Solved { return k; }
    }
    max + 1
}

/// Number of candidates eliminated by T&E(1) from the initial state (lower = harder within depth 2).
pub fn te1_elims(p: &Puzzle) -> u32 {
    let mut s = match State::from_puzzle(p) { Some(s) => s, None => return 0 };
    let mut e = 0;
    te(&mut s, 1, &mut e);
    e
}
