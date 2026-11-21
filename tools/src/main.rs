//! sudoku-tools: command-line helpers for building the puzzle banks.
//!
//! Every command reads puzzles (81 chars, '.' or '0' for empty; anything after the
//! first 81 characters on a line is ignored) from stdin and writes one result line
//! per input line to stdout, in input order. Work is spread over `-t N` threads.
//!
//!   check              puzzle  unique(0/1) minimal(0/1) clues solution
//!   ted [max]          puzzle  T&E-depth (capped at max+1, default max 2)  T&E(1)-eliminations
//!   tier               puzzle  basic-logic tier (1 singles, 2 locked, 3 pairs/X-Wing, 4 wings/Swordfish, 9 harder)
//!   fp                 puzzle  puzzle-fingerprint  solution-grid-fingerprint (hex)
//!   pfp                puzzle-fingerprint only (fast, for building the known-puzzle set)
//!   gen N SEED         generate N random minimal puzzles: puzzle clues tier
//!   vicinity K [known] for each seed, all minimal puzzles in its {-K+K} neighbourhood with
//!                      T&E depth >= 2 that are not in the fingerprint file `known`:
//!                      puzzle depth te1-elims seed-line-number

mod fp;
mod grid;
mod logic;
mod te;

use grid::*;
use std::collections::HashSet;
use std::io::{self, BufRead, BufWriter, Write};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Mutex;

fn par_map<F>(items: &[String], threads: usize, f: F) -> Vec<String>
where
    F: Fn(usize, &str) -> String + Sync,
{
    let next = AtomicUsize::new(0);
    let out: Mutex<Vec<Option<String>>> = Mutex::new(vec![None; items.len()]);
    std::thread::scope(|sc| {
        for _ in 0..threads {
            sc.spawn(|| loop {
                let i = next.fetch_add(1, Ordering::Relaxed);
                if i >= items.len() { break; }
                let r = f(i, &items[i]);
                out.lock().unwrap()[i] = Some(r);
            });
        }
    });
    out.into_inner().unwrap().into_iter().map(|x| x.unwrap()).collect()
}

fn read_lines() -> Vec<String> {
    io::stdin().lock().lines().map_while(Result::ok).filter(|l| !l.trim().is_empty() && !l.starts_with('#')).collect()
}

fn puzzle_of(line: &str) -> Option<Puzzle> { parse(line.get(..81)?) }

fn emit(lines: Vec<String>) {
    let stdout = io::stdout();
    let mut w = BufWriter::new(stdout.lock());
    for l in lines { if !l.is_empty() { writeln!(w, "{}", l).unwrap(); } }
}

/// All minimal puzzles in the {-k+k} vicinity of p (k = 1 or 2), deduplicated by fingerprint.
fn vicinity(p: &Puzzle, k: usize, rng: &mut Rng) -> Vec<Puzzle> {
    let clue_cells: Vec<usize> = (0..81).filter(|&i| p[i] != 0).collect();
    let mut seen: HashSet<u64> = HashSet::new();
    seen.insert(fp::puzzle_fp(p));
    let mut out = Vec::new();
    let removals: Vec<Vec<usize>> = if k == 1 {
        clue_cells.iter().map(|&i| vec![i]).collect()
    } else {
        let mut v = Vec::new();
        for a in 0..clue_cells.len() { for b in a + 1..clue_cells.len() { v.push(vec![clue_cells[a], clue_cells[b]]); } }
        v
    };
    let limit = if k == 1 { 5000 } else { 20000 };
    // dense counters indexed by (cell, digit) -> cell*9 + digit-1, and pairs of those
    let mut c2 = if k == 2 { vec![0u16; 729 * 729] } else { Vec::new() };
    for rem in removals {
        let mut q = *p;
        for &i in &rem { q[i] = 0; }
        let (n, sols) = count_solutions(&q, limit + 1, limit + 1);
        if n > limit || n < 2 { continue; }
        let empty: Vec<usize> = (0..81).filter(|&i| q[i] == 0).collect();
        let mut adds: Vec<Vec<(usize, u8)>> = Vec::new();
        // single additions that pin exactly one solution
        let mut c1 = [0u32; 729];
        for s in &sols { for &i in &empty { c1[i * 9 + s[i] as usize - 1] += 1; } }
        for (id, &c) in c1.iter().enumerate() { if c == 1 { adds.push(vec![(id / 9, (id % 9) as u8 + 1)]); } }
        if k == 2 {
            let mut touched: Vec<usize> = Vec::new();
            for s in &sols {
                let ids: Vec<usize> = empty.iter().map(|&i| i * 9 + s[i] as usize - 1).filter(|&id| c1[id] > 1).collect();
                for a in 0..ids.len() {
                    for b in a + 1..ids.len() {
                        let key = ids[a] * 729 + ids[b];
                        if c2[key] == 0 { touched.push(key); }
                        c2[key] = c2[key].saturating_add(1);
                    }
                }
            }
            for &key in &touched {
                if c2[key] == 1 {
                    let (x, y) = (key / 729, key % 729);
                    adds.push(vec![(x / 9, (x % 9) as u8 + 1), (y / 9, (y % 9) as u8 + 1)]);
                }
                c2[key] = 0;
            }
        }
        for add in adds {
            let mut r = q;
            for &(i, d) in &add { r[i] = d; }
            let mut order: Vec<usize> = (0..81).filter(|&i| r[i] != 0).collect();
            rng.shuffle(&mut order);
            let m = minimize(&r, &order);
            if seen.insert(fp::puzzle_fp(&m)) { out.push(m); }
        }
    }
    out
}

fn main() {
    let mut args: Vec<String> = std::env::args().skip(1).collect();
    let mut threads = std::thread::available_parallelism().map(|n| n.get()).unwrap_or(1);
    if let Some(pos) = args.iter().position(|a| a == "-t") {
        threads = args[pos + 1].parse().expect("-t N");
        args.drain(pos..pos + 2);
    }
    let cmd = args.first().map(String::as_str).unwrap_or("");
    match cmd {
        "check" => {
            let lines = read_lines();
            emit(par_map(&lines, threads, |_, l| match puzzle_of(l) {
                None => format!("{} invalid", l),
                Some(p) => {
                    let sol = solution(&p);
                    let min = sol.is_some() && is_minimal(&p);
                    format!("{} {} {} {} {}", fmt(&p), sol.is_some() as u8, min as u8, clues(&p), sol.map(|s| fmt(&s)).unwrap_or_default())
                }
            }));
        }
        "ted" => {
            let max: u32 = args.get(1).and_then(|s| s.parse().ok()).unwrap_or(2);
            let lines = read_lines();
            emit(par_map(&lines, threads, |_, l| match puzzle_of(l) {
                None => format!("{} invalid", l),
                Some(p) => format!("{} {} {}", fmt(&p), te::depth(&p, max), te::te1_elims(&p)),
            }));
        }
        "tier" => {
            let lines = read_lines();
            emit(par_map(&lines, threads, |_, l| match puzzle_of(l) {
                None => format!("{} invalid", l),
                Some(p) => format!("{} {}", fmt(&p), logic::basic_tier(&p)),
            }));
        }
        "fp" => {
            let lines = read_lines();
            emit(par_map(&lines, threads, |_, l| match puzzle_of(l) {
                None => String::new(),
                Some(p) => {
                    let g = solution(&p).map(|s| fp::grid_fp(&s)).unwrap_or(0);
                    format!("{} {:016x} {:016x}", fmt(&p), fp::puzzle_fp(&p), g)
                }
            }));
        }
        "pfp" => {
            let lines = read_lines();
            emit(par_map(&lines, threads, |_, l| match puzzle_of(l) {
                None => String::new(),
                Some(p) => format!("{:016x}", fp::puzzle_fp(&p)),
            }));
        }
        "selftest" => {
            // fingerprints must be invariant under transformations and (almost always) distinct otherwise
            let lines = read_lines();
            let mut rng = Rng::new(42);
            let (mut bad, mut fps, mut gfps) = (0, HashSet::new(), HashSet::new());
            for l in &lines {
                let p = match puzzle_of(l) { Some(p) => p, None => continue };
                let s = solution(&p).expect("unique");
                let (f, g) = (fp::puzzle_fp(&p), fp::grid_fp(&s));
                for _ in 0..5 {
                    let q = random_transform(&p, &mut rng);
                    let sq = solution(&q).unwrap();
                    if fp::puzzle_fp(&q) != f || fp::grid_fp(&sq) != g || te::depth(&q, 2) != te::depth(&p, 2) { bad += 1; }
                }
                fps.insert(f);
                gfps.insert(g);
            }
            println!("puzzles {} invariance-failures {} distinct-puzzle-fps {} distinct-grid-fps {}", lines.len(), bad, fps.len(), gfps.len());
        }
        "gen" => {
            let n: usize = args[1].parse().unwrap();
            let seed: u64 = args[2].parse().unwrap();
            let idx: Vec<String> = (0..n).map(|i| i.to_string()).collect();
            emit(par_map(&idx, threads, |i, _| {
                let mut rng = Rng::new(seed.wrapping_mul(1_000_003).wrapping_add(i as u64 + 1));
                let p = random_minimal(&mut rng);
                format!("{} {} {}", fmt(&p), clues(&p), logic::basic_tier(&p))
            }));
        }
        "vicinity" => {
            let k: usize = args[1].parse().unwrap();
            let known: HashSet<u64> = match args.get(2) {
                None => HashSet::new(),
                Some(f) => std::fs::read_to_string(f).unwrap().lines().filter_map(|l| u64::from_str_radix(l.trim(), 16).ok()).collect(),
            };
            eprintln!("known fingerprints: {}", known.len());
            let lines = read_lines();
            let global: Mutex<HashSet<u64>> = Mutex::new(HashSet::new());
            emit(par_map(&lines, threads, |n, l| {
                let p = match puzzle_of(l) { Some(p) => p, None => return String::new() };
                let mut rng = Rng::new(n as u64 + 7);
                let mut res = Vec::new();
                for m in vicinity(&p, k, &mut rng) {
                    let f = fp::puzzle_fp(&m);
                    if known.contains(&f) || !global.lock().unwrap().insert(f) { continue; }
                    let d = te::depth(&m, 2);
                    if d >= 2 { res.push(format!("{} {} {} {}", fmt(&m), d, te::te1_elims(&m), n + 1)); }
                }
                eprintln!("seed {} done: {} candidates", n + 1, res.len());
                res.join("\n")
            }));
        }
        _ => {
            eprintln!("usage: sudoku-tools [-t N] check|ted [max]|tier|fp|gen N SEED|vicinity K [knownfp]");
            std::process::exit(2);
        }
    }
}
