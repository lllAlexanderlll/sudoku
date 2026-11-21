//! Core sudoku representation: lookup tables, candidate state with basic
//! propagation (singles), and a backtracking solution counter.

use std::sync::OnceLock;

pub type Puzzle = [u8; 81];
pub const ALL: u16 = 0x1FF;

pub struct Tables {
    pub units: [[usize; 9]; 27],
    pub peers: [[usize; 20]; 81],
}

pub fn row(i: usize) -> usize { i / 9 }
pub fn col(i: usize) -> usize { i % 9 }
pub fn bx(i: usize) -> usize { (i / 27) * 3 + (i % 9) / 3 }

pub fn tables() -> &'static Tables {
    static T: OnceLock<Tables> = OnceLock::new();
    T.get_or_init(|| {
        let mut units = [[0usize; 9]; 27];
        for r in 0..9 {
            for c in 0..9 {
                let i = r * 9 + c;
                units[r][c] = i;
                units[9 + c][r] = i;
                let b = bx(i);
                let k = (r % 3) * 3 + c % 3;
                units[18 + b][k] = i;
            }
        }
        let mut peers = [[0usize; 20]; 81];
        for i in 0..81 {
            let mut n = 0;
            for j in 0..81 {
                if j != i && (row(j) == row(i) || col(j) == col(i) || bx(j) == bx(i)) {
                    peers[i][n] = j;
                    n += 1;
                }
            }
            assert_eq!(n, 20);
        }
        Tables { units, peers }
    })
}

pub fn parse(s: &str) -> Option<Puzzle> {
    let mut p = [0u8; 81];
    let mut n = 0;
    for ch in s.chars() {
        if n == 81 { break; }
        match ch {
            '1'..='9' => { p[n] = ch as u8 - b'0'; n += 1; }
            '.' | '0' => { p[n] = 0; n += 1; }
            _ => return None,
        }
    }
    if n == 81 { Some(p) } else { None }
}

pub fn fmt(p: &Puzzle) -> String {
    p.iter().map(|&v| if v == 0 { '.' } else { (b'0' + v) as char }).collect()
}

pub fn clues(p: &Puzzle) -> usize { p.iter().filter(|&&v| v != 0).count() }

/// Small fast PRNG (xorshift64*), deterministic per seed.
pub struct Rng(u64);
impl Rng {
    pub fn new(seed: u64) -> Self { Rng(seed.wrapping_mul(0x9E3779B97F4A7C15) | 1) }
    pub fn next(&mut self) -> u64 {
        let mut x = self.0;
        x ^= x >> 12; x ^= x << 25; x ^= x >> 27;
        self.0 = x;
        x.wrapping_mul(0x2545F4914F6CDD1D)
    }
    pub fn below(&mut self, n: usize) -> usize { ((self.next() >> 32) as usize * n) >> 32 }
    pub fn shuffle<T>(&mut self, a: &mut [T]) {
        for i in (1..a.len()).rev() { let j = self.below(i + 1); a.swap(i, j); }
    }
}

#[derive(Clone)]
pub struct State {
    pub v: [u8; 81],
    pub c: [u16; 81],
    pub left: u8,
}

impl State {
    pub fn empty() -> State { State { v: [0; 81], c: [ALL; 81], left: 81 } }

    /// Places the givens (eliminating from peers). None if givens conflict.
    pub fn from_puzzle(p: &Puzzle) -> Option<State> {
        let mut s = State::empty();
        for i in 0..81 {
            if p[i] != 0 && !s.assign(i, p[i]) { return None; }
        }
        Some(s)
    }

    /// Sets cell i to digit d and removes d from all peers.
    pub fn assign(&mut self, i: usize, d: u8) -> bool {
        let b = 1u16 << (d - 1);
        if self.c[i] & b == 0 || self.v[i] != 0 { return false; }
        self.v[i] = d;
        self.c[i] = b;
        self.left -= 1;
        for &p in &tables().peers[i] {
            if self.c[p] & b != 0 {
                if self.v[p] != 0 { return false; }
                self.c[p] &= !b;
                if self.c[p] == 0 { return false; }
            }
        }
        true
    }

    pub fn eliminate(&mut self, i: usize, d: u8) -> bool {
        self.c[i] &= !(1u16 << (d - 1));
        self.c[i] != 0
    }

    /// Basic resolution theory: naked and hidden singles to a fixpoint.
    /// Returns false on contradiction.
    pub fn propagate(&mut self) -> bool {
        let t = tables();
        loop {
            let mut changed = false;
            for i in 0..81 {
                if self.v[i] == 0 {
                    let m = self.c[i];
                    if m == 0 { return false; }
                    if m & (m - 1) == 0 {
                        if !self.assign(i, m.trailing_zeros() as u8 + 1) { return false; }
                        changed = true;
                    }
                }
            }
            if self.left == 0 { return true; }
            for u in &t.units {
                let (mut once, mut more, mut placed) = (0u16, 0u16, 0u16);
                for &i in u {
                    if self.v[i] != 0 { placed |= self.c[i]; } else { more |= once & self.c[i]; once |= self.c[i]; }
                }
                if (once | placed) != ALL { return false; }
                let mut hidden = once & !more & !placed;
                while hidden != 0 {
                    let b = hidden & hidden.wrapping_neg();
                    hidden ^= b;
                    for &i in u {
                        if self.v[i] == 0 && self.c[i] & b != 0 {
                            if !self.assign(i, b.trailing_zeros() as u8 + 1) { return false; }
                            changed = true;
                            break;
                        }
                    }
                }
            }
            if !changed { return true; }
        }
    }

    pub fn solved(&self) -> bool { self.left == 0 }
}

fn best_cell(s: &State) -> usize {
    let mut best = 81;
    let mut bn = 10;
    for i in 0..81 {
        if s.v[i] == 0 {
            let n = s.c[i].count_ones();
            if n < bn { bn = n; best = i; if n == 2 { break; } }
        }
    }
    best
}

fn rec(mut s: State, limit: usize, cnt: &mut usize, sols: &mut Vec<Puzzle>, store: usize, rng: &mut Option<&mut Rng>) {
    if !s.propagate() { return; }
    if s.solved() {
        *cnt += 1;
        if sols.len() < store { sols.push(s.v); }
        return;
    }
    let i = best_cell(&s);
    let mut ds: [u8; 9] = [0; 9];
    let mut n = 0;
    let mut m = s.c[i];
    while m != 0 { ds[n] = m.trailing_zeros() as u8 + 1; m &= m - 1; n += 1; }
    if let Some(r) = rng.as_mut() { r.shuffle(&mut ds[..n]); }
    for &d in &ds[..n] {
        let mut t = s.clone();
        if t.assign(i, d) { rec(t, limit, cnt, sols, store, rng); }
        if *cnt >= limit { return; }
    }
}

/// Counts solutions up to `limit`, storing up to `store` of them.
pub fn count_solutions(p: &Puzzle, limit: usize, store: usize) -> (usize, Vec<Puzzle>) {
    let mut cnt = 0;
    let mut sols = Vec::new();
    if let Some(s) = State::from_puzzle(p) { rec(s, limit, &mut cnt, &mut sols, store, &mut None); }
    (cnt, sols)
}

pub fn unique(p: &Puzzle) -> bool { count_solutions(p, 2, 0).0 == 1 }

pub fn solution(p: &Puzzle) -> Option<Puzzle> {
    let (n, s) = count_solutions(p, 2, 1);
    if n == 1 { Some(s[0]) } else { None }
}

pub fn random_grid(rng: &mut Rng) -> Puzzle {
    let mut cnt = 0;
    let mut sols = Vec::new();
    rec(State::empty(), 1, &mut cnt, &mut sols, 1, &mut Some(rng));
    sols[0]
}

/// True if the puzzle has a unique solution and no clue can be removed.
pub fn is_minimal(p: &Puzzle) -> bool {
    if !unique(p) { return false; }
    let mut q = *p;
    for i in 0..81 {
        if p[i] != 0 {
            q[i] = 0;
            let u = unique(&q);
            q[i] = p[i];
            if u { return false; }
        }
    }
    true
}

/// Removes clues in the given order while uniqueness is kept. Result is minimal.
pub fn minimize(p: &Puzzle, order: &[usize]) -> Puzzle {
    let mut q = *p;
    for &i in order {
        if q[i] != 0 {
            let d = q[i];
            q[i] = 0;
            if !unique(&q) { q[i] = d; }
        }
    }
    q
}

pub fn random_minimal(rng: &mut Rng) -> Puzzle {
    let g = random_grid(rng);
    let mut order: Vec<usize> = (0..81).collect();
    rng.shuffle(&mut order);
    minimize(&g, &order)
}

/// Applies a random validity-preserving transformation (digits, bands, rows, stacks, columns, transpose).
pub fn random_transform(p: &Puzzle, rng: &mut Rng) -> Puzzle {
    let mut digits: Vec<u8> = (1..=9).collect();
    rng.shuffle(&mut digits);
    let perm = |rng: &mut Rng| { let mut a = [0usize, 1, 2]; rng.shuffle(&mut a); a };
    let mut rows = Vec::new();
    let mut cols = Vec::new();
    for b in perm(rng) { for r in perm(rng) { rows.push(b * 3 + r); } }
    for s in perm(rng) { for c in perm(rng) { cols.push(s * 3 + c); } }
    let tr = rng.below(2) == 1;
    let mut out = [0u8; 81];
    for r in 0..9 {
        for c in 0..9 {
            let (sr, sc) = if tr { (cols[c], rows[r]) } else { (rows[r], cols[c]) };
            let v = p[sr * 9 + sc];
            out[r * 9 + c] = if v == 0 { 0 } else { digits[v as usize - 1] };
        }
    }
    out
}
