//! A small human-style solver used only as a cheap pre-filter before Sudoku Explainer.
//! Techniques: singles, locked candidates, naked/hidden subsets (2-4),
//! X-Wing/Swordfish, XY-Wing, XYZ-Wing. Roughly covers SER <= 4.4.

use crate::grid::{bx, col, row, tables, Puzzle, State};

fn bits(m: u16) -> impl Iterator<Item = u8> {
    (0..9u8).filter(move |d| m & (1 << d) != 0)
}

fn combos(n: usize, k: usize) -> Vec<Vec<usize>> {
    let mut out = Vec::new();
    let mut cur = Vec::new();
    fn go(start: usize, n: usize, k: usize, cur: &mut Vec<usize>, out: &mut Vec<Vec<usize>>) {
        if cur.len() == k { out.push(cur.clone()); return; }
        for i in start..n { cur.push(i); go(i + 1, n, k, cur, out); cur.pop(); }
    }
    go(0, n, k, &mut cur, &mut out);
    out
}

fn rm(s: &mut State, i: usize, mask: u16, ch: &mut bool) {
    if s.v[i] == 0 && s.c[i] & mask != 0 { s.c[i] &= !mask; *ch = true; }
}

fn locked(s: &mut State) -> bool {
    let t = tables();
    let mut ch = false;
    for b in 0..9 {
        for d in 0..9u8 {
            let bit = 1u16 << d;
            let cells: Vec<usize> = t.units[18 + b].iter().copied().filter(|&i| s.v[i] == 0 && s.c[i] & bit != 0).collect();
            if cells.len() < 2 { continue; }
            if cells.iter().all(|&i| row(i) == row(cells[0])) {
                for &j in &t.units[row(cells[0])] { if bx(j) != b { rm(s, j, bit, &mut ch); } }
            }
            if cells.iter().all(|&i| col(i) == col(cells[0])) {
                for &j in &t.units[9 + col(cells[0])] { if bx(j) != b { rm(s, j, bit, &mut ch); } }
            }
        }
    }
    for u in 0..18 {
        for d in 0..9u8 {
            let bit = 1u16 << d;
            let cells: Vec<usize> = t.units[u].iter().copied().filter(|&i| s.v[i] == 0 && s.c[i] & bit != 0).collect();
            if cells.len() < 2 { continue; }
            let b = bx(cells[0]);
            if cells.iter().all(|&i| bx(i) == b) {
                for &j in &t.units[18 + b] { if !t.units[u].contains(&j) { rm(s, j, bit, &mut ch); } }
            }
        }
    }
    ch
}

fn subsets(s: &mut State, k: usize) -> bool {
    let t = tables();
    let mut ch = false;
    for u in &t.units {
        let empty: Vec<usize> = u.iter().copied().filter(|&i| s.v[i] == 0).collect();
        if empty.len() <= k { continue; }
        // naked
        for cmb in combos(empty.len(), k) {
            let m = cmb.iter().fold(0u16, |a, &x| a | s.c[empty[x]]);
            if m.count_ones() as usize == k {
                for (x, &j) in empty.iter().enumerate() { if !cmb.contains(&x) { rm(s, j, m, &mut ch); } }
            }
        }
        // hidden
        let mut free = 0u16;
        for &i in &empty { free |= s.c[i]; }
        let ds: Vec<u8> = bits(free).collect();
        if ds.len() <= k { continue; }
        for cmb in combos(ds.len(), k) {
            let dm = cmb.iter().fold(0u16, |a, &x| a | (1 << ds[x]));
            let cells: Vec<usize> = empty.iter().copied().filter(|&i| s.c[i] & dm != 0).collect();
            if cells.len() == k && cmb.iter().all(|&x| cells.iter().any(|&i| s.c[i] & (1 << ds[x]) != 0)) {
                for &i in &cells { rm(s, i, !dm & 0x1FF, &mut ch); }
            }
        }
    }
    ch
}

fn fish(s: &mut State, k: usize) -> bool {
    let mut ch = false;
    for d in 0..9u8 {
        let bit = 1u16 << d;
        for transposed in [false, true] {
            let cell = |a: usize, b: usize| if transposed { b * 9 + a } else { a * 9 + b };
            // base lines with 2..=k candidates
            let mut lines = Vec::new();
            for a in 0..9 {
                let mut m = 0u16;
                for b in 0..9 { let i = cell(a, b); if s.v[i] == 0 && s.c[i] & bit != 0 { m |= 1 << b; } }
                if m != 0 && m.count_ones() as usize <= k { lines.push((a, m)); }
            }
            if lines.len() < k { continue; }
            for cmb in combos(lines.len(), k) {
                let cover = cmb.iter().fold(0u16, |acc, &x| acc | lines[x].1);
                if cover.count_ones() as usize != k { continue; }
                let base: Vec<usize> = cmb.iter().map(|&x| lines[x].0).collect();
                for b in bits(cover) {
                    for a in 0..9 { if !base.contains(&a) { rm(s, cell(a, b as usize), bit, &mut ch); } }
                }
            }
        }
    }
    ch
}

fn sees(a: usize, b: usize) -> bool { a != b && (row(a) == row(b) || col(a) == col(b) || bx(a) == bx(b)) }

fn wings(s: &mut State) -> bool {
    let mut ch = false;
    let empty: Vec<usize> = (0..81).filter(|&i| s.v[i] == 0).collect();
    for &p in &empty {
        let pc = s.c[p].count_ones();
        if pc != 2 && pc != 3 { continue; }
        let pincers: Vec<usize> = empty.iter().copied().filter(|&q| sees(p, q) && s.c[q].count_ones() == 2).collect();
        for x in 0..pincers.len() {
            for y in x + 1..pincers.len() {
                let (a, b) = (pincers[x], pincers[y]);
                let (ma, mb, mp) = (s.c[a], s.c[b], s.c[p]);
                if pc == 2 {
                    // XY-Wing: p=xy, a=xz, b=yz
                    if ma == mb || (ma & mp).count_ones() != 1 || (mb & mp).count_ones() != 1 || (ma & mb & mp) != 0 { continue; }
                    let z = ma & mb;
                    if z.count_ones() != 1 { continue; }
                    for &j in &empty { if j != p && sees(j, a) && sees(j, b) { rm(s, j, z, &mut ch); } }
                } else {
                    // XYZ-Wing: p=xyz, a=xz, b=yz (subsets of p, distinct)
                    if ma == mb || ma & !mp != 0 || mb & !mp != 0 { continue; }
                    let z = ma & mb;
                    if z.count_ones() != 1 { continue; }
                    for &j in &empty { if j != p && sees(j, a) && sees(j, b) && sees(j, p) { rm(s, j, z, &mut ch); } }
                }
            }
        }
    }
    ch
}

/// Returns the highest technique tier used: 1 singles, 2 locked candidates,
/// 3 pairs/triples/X-Wing, 4 quads/Swordfish/wings; 9 if the basic set does not solve it.
pub fn basic_tier(p: &Puzzle) -> u8 {
    let mut s = match State::from_puzzle(p) { Some(s) => s, None => return 0 };
    let mut tier = 1u8;
    loop {
        if !s.propagate() { return 0; }
        if s.solved() { return tier; }
        if locked(&mut s) { tier = tier.max(2); continue; }
        if subsets(&mut s, 2) || fish(&mut s, 2) || subsets(&mut s, 3) { tier = tier.max(3); continue; }
        if fish(&mut s, 3) || wings(&mut s) || subsets(&mut s, 4) { tier = tier.max(4); continue; }
        return 9;
    }
}
