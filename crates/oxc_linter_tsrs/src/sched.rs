//! Which checker lints which file.
//!
//! tsrs assigns every program file to one checker by import-graph locality (tsrs `notes/mem-assignment.md`). A
//! checker that lints only its own files resolves far fewer types than one that pulls from a shared queue, where
//! every checker ends up resolving the types of nearly every module. The assignment balances an estimate of the
//! cost, so one checker can still be left as the tail: here each checker lints its own files in program order, and
//! a checker that runs out takes not-yet-started files from the back of the checker with the most work left.
//!
//! `TSRSLINT_SCHEDULE=locality` turns stealing off (every file on its assigned checker; output byte-identical run to
//! run). `TSRSLINT_CHECKER_STATS=1` prints per-checker file counts, wall and thread CPU time to stderr.

use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Mutex, OnceLock};
use std::time::{Duration, Instant};

use tsrs_ast::SourceFile;
use tsrs_checker::Checker;
use tsrs_compiler::Program;
use tsrs_core::P;

fn steal_enabled() -> bool {
    static S: OnceLock<bool> = OnceLock::new();
    *S.get_or_init(|| std::env::var("TSRSLINT_SCHEDULE").map_or(true, |v| v != "locality"))
}

pub(crate) fn stats_enabled() -> bool {
    static S: OnceLock<bool> = OnceLock::new();
    *S.get_or_init(|| {
        std::env::var("TSRSLINT_CHECKER_STATS").is_ok_and(|v| !v.is_empty() && v != "0")
    })
}

/// Checkers per program: `TSRSLINT_CHECKERS=<n>`, else 6 with at least 8 hardware threads, else tsrs's default (4).
/// Every checker resolves the types its files need on its own, so each one adds CPU and memory: on the monolith 6
/// checkers cost ~17% more instructions and ~1 GiB more peak RSS than 4, which only pays off with spare cores.
pub(crate) fn checkers() -> Option<i64> {
    static N: OnceLock<Option<i64>> = OnceLock::new();
    *N.get_or_init(|| {
        if let Some(n) =
            std::env::var("TSRSLINT_CHECKERS").ok().and_then(|v| v.parse().ok()).filter(|&n| n > 0)
        {
            return Some(n);
        }
        let threads = std::thread::available_parallelism().map_or(1, |n| n.get());
        (threads >= 8).then_some(6)
    })
}

/// One checker's files (indices into the lint file list) in program order. The owner takes from the front, other
/// checkers from the back. `range` packs the next front position (low 32 bits) and the back end (high 32 bits,
/// exclusive); `prefix[k]` is the node count of `files[..k]`, the estimate of the work left.
struct FileQueue {
    files: Vec<u32>,
    prefix: Vec<u64>,
    range: AtomicU64,
}

const LOW: u64 = u32::MAX as u64;

impl FileQueue {
    fn new(files: Vec<u32>, weight: impl Fn(u32) -> u64) -> FileQueue {
        let mut prefix = Vec::with_capacity(files.len() + 1);
        let mut sum = 0;
        prefix.push(0);
        for &i in &files {
            sum += weight(i);
            prefix.push(sum);
        }
        let len = files.len() as u64;
        FileQueue { files, prefix, range: AtomicU64::new(len << 32) }
    }

    fn remaining(&self) -> u64 {
        // Relaxed: an estimate for choosing a queue to steal from; a stale value only changes that choice.
        let r = self.range.load(Ordering::Relaxed);
        let (front, back) = ((r & LOW) as usize, (r >> 32) as usize);
        if front < back { self.prefix[back] - self.prefix[front] } else { 0 }
    }

    fn take(&self, front: bool) -> Option<usize> {
        // Relaxed (here and in the CAS): `range` only hands out indices into `files`, which never changes after
        // new(), so no other data is published through it; the CAS alone gives each position to one taker.
        let mut r = self.range.load(Ordering::Relaxed);
        loop {
            if (r & LOW) >= (r >> 32) {
                return None;
            }
            let new = if front { r + 1 } else { r - (1 << 32) };
            // Relaxed: see above.
            match self.range.compare_exchange_weak(r, new, Ordering::Relaxed, Ordering::Relaxed) {
                Ok(_) => break,
                Err(cur) => r = cur,
            }
        }
        let pos = if front { r & LOW } else { (r >> 32) - 1 };
        Some(self.files[pos as usize] as usize)
    }
}

/// The next file for checker `me` and whether it belongs to another checker.
fn next(queues: &[FileQueue], me: usize, steal: bool) -> Option<(usize, bool)> {
    if let Some(i) = queues[me].take(true) {
        return Some((i, false));
    }
    if !steal {
        return None;
    }
    loop {
        let (victim, left) = queues
            .iter()
            .enumerate()
            .map(|(c, q)| (c, q.remaining()))
            .max_by_key(|&(c, left)| (left, std::cmp::Reverse(c)))?;
        if left == 0 {
            return None;
        }
        if let Some(i) = queues[victim].take(false) {
            return Some((i, true));
        }
    }
}

#[derive(Default, Clone, Copy)]
struct CheckerStats {
    files: usize,
    stolen: usize,
    wall: f64,
    cpu: f64,
}

#[derive(Default)]
struct Totals {
    lint_wall: f64,
    cpu_max: f64,
    cpu_sum: f64,
}

static TOTALS: Mutex<Totals> = Mutex::new(Totals { lint_wall: 0.0, cpu_max: 0.0, cpu_sum: 0.0 });

/// Runs `lint(state, checker, i)` for every index `i` of `todo` (indices into `files`, in program order) on one
/// thread per checker of `program`. `init` creates a thread's state before its first file and `done` gets it back.
pub(crate) fn for_each_file<S>(
    program: &'static Program,
    files: &[P<SourceFile>],
    todo: &[usize],
    init: impl Fn() -> S + Sync,
    lint: impl Fn(&mut S, &mut Checker, usize) + Sync,
    done: impl Fn(S) + Sync,
) {
    let start = Instant::now();
    let checkers = program.checker_count();
    let mut per: Vec<Vec<u32>> = vec![Vec::new(); checkers];
    for &i in todo {
        let c = program.checker_index_of_file(files[i]).filter(|&c| c < checkers).unwrap_or(0);
        per[c].push(i as u32);
    }
    let queues: Vec<FileQueue> = per
        .into_iter()
        .map(|f| FileQueue::new(f, |i| files[i as usize].node_count.get() as u64))
        .collect();
    let steal = steal_enabled();
    let stats = stats_enabled();
    let per_checker: Mutex<Vec<CheckerStats>> = Mutex::new(vec![CheckerStats::default(); checkers]);
    program.for_each_checker_parallel(|idx, checker| {
        let wall = Instant::now();
        let cpu = if stats { thread_cpu_seconds() } else { 0.0 };
        let mut s = CheckerStats::default();
        let mut state = init();
        while let Some((i, stolen)) = next(&queues, idx, steal) {
            lint(&mut state, checker, i);
            s.files += 1;
            s.stolen += stolen as usize;
        }
        done(state);
        if stats {
            s.cpu = thread_cpu_seconds() - cpu;
            s.wall = wall.elapsed().as_secs_f64();
            per_checker.lock().unwrap()[idx] = s;
        }
    });
    if stats {
        report(steal, todo.len(), start.elapsed(), &per_checker.into_inner().unwrap());
    }
}

fn report(steal: bool, files: usize, lint_wall: Duration, per: &[CheckerStats]) {
    use std::fmt::Write;
    let lint_wall = lint_wall.as_secs_f64();
    let mut out = format!(
        "tsrslint checker stats: {}, {} checkers, {files} files, lint wall {lint_wall:.2}s\n",
        if steal { "steal" } else { "locality" },
        per.len()
    );
    for (i, s) in per.iter().enumerate() {
        let _ = writeln!(
            out,
            "  checker {i}: {} files ({} stolen), wall {:.2}s, cpu {:.2}s",
            s.files, s.stolen, s.wall, s.cpu
        );
    }
    let cpu_max = per.iter().map(|s| s.cpu).fold(0.0, f64::max);
    let cpu_sum: f64 = per.iter().map(|s| s.cpu).sum();
    let mean = cpu_sum / per.len().max(1) as f64;
    let _ = writeln!(
        out,
        "  cpu max {cpu_max:.2}s, mean {mean:.2}s (max/mean {:.2}), sum {cpu_sum:.2}s",
        if mean > 0.0 { cpu_max / mean } else { 0.0 }
    );
    eprint!("{out}");
    let mut t = TOTALS.lock().unwrap();
    t.lint_wall += lint_wall;
    t.cpu_max += cpu_max;
    t.cpu_sum += cpu_sum;
}

/// Prints the run totals: `prelint` is everything outside the lint phases (tsconfig assignment, program creation,
/// binding, checker creation, output).
pub(crate) fn report_totals(run_wall: Duration) {
    if !stats_enabled() {
        return;
    }
    let t = TOTALS.lock().unwrap();
    let run_wall = run_wall.as_secs_f64();
    eprintln!(
        "tsrslint checker stats total: wall {run_wall:.2}s, prelint {:.2}s, lint wall {:.2}s, lint cpu max {:.2}s, lint cpu sum {:.2}s",
        run_wall - t.lint_wall,
        t.lint_wall,
        t.cpu_max,
        t.cpu_sum,
    );
}

/// CPU time of the calling thread: unlike wall time it does not grow while other processes hold the cores.
#[cfg(unix)]
fn thread_cpu_seconds() -> f64 {
    #[repr(C)]
    struct Timespec {
        tv_sec: i64,
        tv_nsec: i64,
    }
    extern "C" {
        fn clock_gettime(clock_id: i32, tp: *mut Timespec) -> i32;
    }
    #[cfg(target_os = "macos")]
    const CLOCK_THREAD_CPUTIME_ID: i32 = 16;
    #[cfg(not(target_os = "macos"))]
    const CLOCK_THREAD_CPUTIME_ID: i32 = 3;
    let mut ts = Timespec { tv_sec: 0, tv_nsec: 0 };
    // SAFETY: clock_gettime writes one timespec through the valid pointer.
    if unsafe { clock_gettime(CLOCK_THREAD_CPUTIME_ID, &raw mut ts) } != 0 {
        return 0.0;
    }
    ts.tv_sec as f64 + ts.tv_nsec as f64 * 1e-9
}

#[cfg(not(unix))]
fn thread_cpu_seconds() -> f64 {
    0.0
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Owners taking from the front and thieves from the back of the same queues must hand out every file exactly
    /// once: a lost index is a file that is never linted (missing diagnostics), a repeated one reports twice.
    #[test]
    fn every_file_is_taken_exactly_once() {
        let sizes = [5000usize, 0, 20000, 300];
        let mut base = 0u32;
        let queues: Vec<FileQueue> = sizes
            .iter()
            .map(|&n| {
                let files: Vec<u32> = (base..base + n as u32).collect();
                base += n as u32;
                FileQueue::new(files, |i| u64::from(i % 7 + 1))
            })
            .collect();
        let taken: Mutex<Vec<(usize, bool)>> = Mutex::new(Vec::new());
        std::thread::scope(|s| {
            for me in 0..queues.len() {
                let (queues, taken) = (&queues, &taken);
                s.spawn(move || {
                    let mut mine = Vec::new();
                    while let Some(t) = next(queues, me, true) {
                        mine.push(t);
                    }
                    taken.lock().unwrap().extend(mine);
                });
            }
        });
        let taken = taken.into_inner().unwrap();
        let mut ids: Vec<usize> = taken.iter().map(|&(i, _)| i).collect();
        ids.sort_unstable();
        assert_eq!(ids, (0..base as usize).collect::<Vec<_>>());
        assert!(queues.iter().all(|q| q.remaining() == 0));
    }

    /// Without stealing a checker lints exactly its own files, in program order (the order is load-bearing for
    /// printed types, tsrs notes/perf-balance.md).
    #[test]
    fn locality_keeps_own_files_in_order() {
        let queues = vec![FileQueue::new(vec![0, 2, 4], |_| 1), FileQueue::new(vec![1, 3], |_| 1)];
        let order: Vec<(usize, bool)> = std::iter::from_fn(|| next(&queues, 0, false)).collect();
        assert_eq!(order, vec![(0, false), (2, false), (4, false)]);
        assert_eq!(queues[1].remaining(), 2);
    }
}
